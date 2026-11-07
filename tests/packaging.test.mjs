import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,existsSync} from 'node:fs';
import {posix} from 'node:path';

test('installed companion includes manifest entry points and their local dependencies',()=>{
 const configUrl=new URL('../apps/desktop/src-tauri/tauri.conf.json',import.meta.url);
 const config=JSON.parse(readFileSync(configUrl,'utf8'));
 const bundled=new Map(Object.entries(config.bundle.resources)
  .filter(([,target])=>target.startsWith('browser-extension/'))
  .map(([source,target])=>[target.slice('browser-extension/'.length),new URL(source,configUrl)]));
 const loaded=new Set(),pending=['manifest.json'];
 function requireFile(file,from){
  assert.ok(!file.startsWith('../')&&!posix.isAbsolute(file),`Unsafe companion reference ${file}`);
  assert.ok(bundled.has(file),`Installer omits ${file} required by ${from}`);
  if(!loaded.has(file))pending.push(file);
 }
 function dependency(from,relative){
  if(/^(?:[a-z]+:|\/\/|#)/i.test(relative))return;
  requireFile(posix.normalize(posix.join(posix.dirname(from),relative)),from);
 }
 while(pending.length){
  const file=pending.pop();if(loaded.has(file))continue;loaded.add(file);
  const source=bundled.get(file);assert.ok(source,`Installer omits ${file}`);
  assert.ok(existsSync(source),`Bundled companion source missing: ${file}`);
  if(file==='manifest.json'){
   const manifest=JSON.parse(readFileSync(source,'utf8'));
   const entryPoints=[manifest.background?.service_worker,manifest.action?.default_popup,
    ...Object.values(manifest.icons||{}),...Object.values(manifest.action?.default_icon||{}),
    ...(manifest.content_scripts||[]).flatMap(s=>[...(s.js||[]),...(s.css||[])]),
    ...(manifest.web_accessible_resources||[]).flatMap(r=>r.resources)];
   for(const entry of entryPoints.filter(Boolean))requireFile(entry,'manifest.json');
  }else if(file.endsWith('.js')){
   const text=readFileSync(source,'utf8');
   for(const match of text.matchAll(/\bfrom\s*['"]([^'"]+)['"]|\bimport\s*['"]([^'"]+)['"]|\bimport\s*\(\s*['"]([^'"]+)['"]\s*\)/g)){
    const relative=match[1]||match[2]||match[3];if(relative.startsWith('.'))dependency(file,relative);
   }
  }else if(file.endsWith('.html')){
   for(const match of readFileSync(source,'utf8').matchAll(/\b(?:src|href)\s*=\s*['"]([^'"]+)['"]/g))dependency(file,match[1]);
  }
 }
 assert.ok(loaded.has('src/background.js')&&loaded.has('src/video.js'),'Companion worker and player must be checked');
});
