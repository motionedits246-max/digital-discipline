// Exercise exactly the Chromium native-messaging protocol against the installed host.
// Usage: node scripts/test-native-bridge.mjs Brave
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
const browser=['Brave','Chrome','Edge'].includes(process.argv[2])?process.argv[2]:'Brave';
const manifestPath=path.join(process.env.LOCALAPPDATA||'', 'DigitalDiscipline',`com.digitaldiscipline.bridge.${browser}.json`);
if(!fs.existsSync(manifestPath)){console.error('Pairing manifest not found:',manifestPath);process.exit(1);}
const manifest=JSON.parse(fs.readFileSync(manifestPath,'utf8'));
if(!path.isAbsolute(manifest.path)||!fs.existsSync(manifest.path)){console.error('Native host executable not found:',manifest.path);process.exit(1);}
const processHost=spawn(manifest.path,['--native-host'],{stdio:['pipe','pipe','pipe'],windowsHide:true});
const message=Buffer.from(JSON.stringify({op:'sync'}));
const header=Buffer.alloc(4);header.writeUInt32LE(message.length,0);
processHost.stdin.write(Buffer.concat([header,message]));
const chunks=[];let size=0;let received=false;
const timeout=setTimeout(()=>{console.error('No native reply after 8 seconds. Host is missing, crashed, or blocked.');processHost.kill();process.exitCode=2;},8000);
processHost.stdout.on('data',buf=>{
 chunks.push(buf);size+=buf.length;const combined=Buffer.concat(chunks);
 if(size<4)return;
 const payloadLength=combined.readUInt32LE(0);
 if(payloadLength>1048576){console.error('Invalid native framing');processHost.kill();process.exitCode=3;return;}
 if(size>=4+payloadLength){received=true;clearTimeout(timeout);try{const response=JSON.parse(combined.subarray(4,4+payloadLength).toString()); console.log('Native bridge reply:',{ok:response.ok===true,error:response.error||null,blockCount:Array.isArray(response.blocks)?response.blocks.length:null});if(response.ok!==true)process.exitCode=4;}catch(e){console.error('Invalid JSON response',e);process.exitCode=5;}processHost.stdin.end();processHost.kill();}
});
processHost.stderr.on('data',buf=>{ // Exclude any tokens, file content, or paths from printed native stderr.
 console.error('Native host stderr reported',buf.length,'bytes.');
});
processHost.on('error',err=>{console.error('Could not start native host:',err.message);clearTimeout(timeout);process.exitCode=6;});
processHost.on('exit',(code)=>{if(!received){clearTimeout(timeout);console.error('Host exited before sending a framed reply. Exit code:',code);process.exitCode=7;}});
