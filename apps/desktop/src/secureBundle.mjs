// Portable encryption. Browser WebCrypto only. The passphrase never reaches the
// Rust agent, browser storage, logs, external services or the exported file.
const ITERATIONS = 350000;
const coder = new TextEncoder();
const decode = new TextDecoder('utf-8',{fatal:true});
function b64(bytes) {
  let out='';
  for(let i=0;i<bytes.length;i+=8192)out+=String.fromCharCode(...bytes.subarray(i,i+8192));
  return btoa(out);
}
function unb64(value) {
  if(typeof value!=='string'||value.length>650000||!/^[a-zA-Z0-9+/]*={0,2}$/.test(value))throw new Error('Invalid bundle encoding');
  return Uint8Array.from(atob(value),v=>v.charCodeAt(0));
}
async function derive(pass,salt) {
  if(typeof pass!=='string'||pass.length<12||pass.length>300)throw new Error('Use a 12+ character passphrase, up to 300 characters');
  const base=await crypto.subtle.importKey('raw',coder.encode(pass),'PBKDF2',false,['deriveKey']);
  return crypto.subtle.deriveKey({name:'PBKDF2',salt,iterations:ITERATIONS,hash:'SHA-256'},base,{name:'AES-GCM',length:256},false,['encrypt','decrypt']);
}
export async function sealBundle(payload,passphrase) {
  if(!crypto?.subtle)throw new Error('WebCrypto is unavailable on this device');
  const json=JSON.stringify(payload);
  if(json.length>390000)throw new Error('Device export too large');
  const salt=crypto.getRandomValues(new Uint8Array(16));
  const iv=crypto.getRandomValues(new Uint8Array(12));
  const key=await derive(passphrase,salt);
  const data=await crypto.subtle.encrypt({name:'AES-GCM',iv},key,coder.encode(json));
  return JSON.stringify({format:'dd-encrypted-bundle-v1',kdf:'PBKDF2-SHA256',iterations:ITERATIONS,cipher:'AES-256-GCM',salt:b64(salt),iv:b64(iv),ciphertext:b64(new Uint8Array(data))});
}
export async function openBundle(encoded,passphrase) {
  if(typeof encoded!=='string'||encoded.length>650000)throw new Error('Encrypted file too large');
  let object;
  try{object=JSON.parse(encoded);}catch{throw new Error('Not a Digital Discipline bundle');}
  if(object?.format!=='dd-encrypted-bundle-v1'||object.kdf!=='PBKDF2-SHA256'||object.iterations!==ITERATIONS||object.cipher!=='AES-256-GCM')throw new Error('Unrecognized or unsupported encryption format');
  const salt=unb64(object.salt);const iv=unb64(object.iv);
  if(salt.length!==16||iv.length!==12)throw new Error('Invalid salt or IV');
  const ciphertext=unb64(object.ciphertext);
  const key=await derive(passphrase,salt);
  let plaintext;
  try{plaintext=await crypto.subtle.decrypt({name:'AES-GCM',iv},key,ciphertext);}catch{throw new Error('Incorrect passphrase or damaged bundle');}
  let payload;
  try{payload=JSON.parse(decode.decode(plaintext));}catch{throw new Error('Invalid decrypted data');}
  if(payload?.schema!=='dd-device-export-v1'||typeof payload.source_device_id!=='string'||!Array.isArray(payload.records))throw new Error('Unexpected device data');
  return payload;
}
