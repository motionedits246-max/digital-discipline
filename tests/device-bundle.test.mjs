import {test} from 'node:test';
import {strict as assert} from 'node:assert';
import {webcrypto} from 'node:crypto';
import {sealBundle,openBundle} from '../apps/desktop/src/secureBundle.mjs';
if(!globalThis.crypto)globalThis.crypto=webcrypto;
const sample={schema:'dd-device-export-v1',source_device_id:'11111111-1111-4111-8111-111111111111',source_device_name:'Laptop',records:[{session_id:'22222222-2222-4222-8222-222222222222',start_ms:1,end_ms:2,app:'cursor.exe',idle:false,domain:null}]};
test('AES-GCM round trips, does not expose plaintext', async()=>{
 const protectedText=await sealBundle(sample,'temporary passphrase for a private device');
 assert.ok(protectedText.includes('dd-encrypted-bundle-v1'));
 assert.equal(protectedText.includes('cursor.exe'),false);
 assert.deepEqual(await openBundle(protectedText,'temporary passphrase for a private device'),sample);
});
test('wrong secret and manipulated ciphertext are rejected',async()=>{
 const text=await sealBundle(sample,'temporary passphrase for a private device');
 await assert.rejects(openBundle(text,'an entirely different passphrase'));
 const blob=JSON.parse(text);blob.ciphertext='AA'+blob.ciphertext.slice(2);
 await assert.rejects(openBundle(JSON.stringify(blob),'temporary passphrase for a private device'));
});
test('cannot downgrade KDF work factor',async()=>{
 const json=JSON.parse(await sealBundle(sample,'temporary passphrase for a private device'));
 json.iterations=1;
 await assert.rejects(openBundle(JSON.stringify(json),'temporary passphrase for a private device'));
});
