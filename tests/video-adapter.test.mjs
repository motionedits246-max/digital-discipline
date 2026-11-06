import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
const core=readFileSync('extension/src/playback-model.js','utf8'),adapter=readFileSync('extension/src/video.js','utf8');
async function fixture(url='https://www.youtube.com/shorts/Example123'){
 let now=0,focused=true,blocks=[];const intervals=[],messages=[],events={},playerEvents={};
 const player={tagName:'VIDEO',dataset:{},clientWidth:640,clientHeight:360,currentTime:0,duration:10,playbackRate:1,paused:false,seeking:false,readyState:4,loop:false,autoplay:true,attrs:{autoplay:''},
  getBoundingClientRect:()=>({width:640,height:360,bottom:360,top:0,left:0,right:640}),addEventListener:(k,v)=>playerEvents[k]=v,
  pause(){this.paused=true;},getAttribute(k){return this.attrs[k]??null;},removeAttribute(k){delete this.attrs[k];},setAttribute(k,v){this.attrs[k]=v;}};
 const document={visibilityState:'visible',hasFocus:()=>focused,querySelectorAll:()=>[player],addEventListener:(k,v)=>events[k]=v,
  documentElement:{appendChild(){}},createElement:()=>({style:{},setAttribute(){},append(){},addEventListener(){},remove(){}})};
 const context=vm.createContext({document,location:new URL(url),innerHeight:800,innerWidth:1200,performance:{now:()=>now},crypto,URLSearchParams,
  setInterval:fn=>intervals.push(fn),window:{addEventListener(){}},Date:class extends Date{static now(){return now;}},
  chrome:{storage:{local:{get:async()=>({collectionEnabled:true})},onChanged:{addListener(){}}},runtime:{onMessage:{addListener(){}},sendMessage:async m=>{messages.push(m);return m.op==='checkBlocked'?{blocks}:{};}}}});
 vm.runInContext(core,context);vm.runInContext(adapter,context);await new Promise(r=>setImmediate(r));
 return {player,messages,context,document,events,playerEvents,setFocused:v=>focused=v,
  async tick(time,media,opts={}){now=time;Object.assign(player,{currentTime:media},opts);for(const fn of intervals)fn();await new Promise(r=>setImmediate(r));},
  async policy(value){blocks=value;events.visibilitychange();await new Promise(r=>setImmediate(r));},
  plays:()=>messages.filter(m=>m.kind==='video.qualified_play')};
}
test('actual Shorts adapter requires visible progressing media, then counts once',async()=>{
 const f=await fixture();await f.tick(1000,0);await f.tick(2000,1,{paused:true});await f.tick(3000,2,{paused:false,readyState:1});f.setFocused(false);await f.tick(4000,3,{readyState:4});f.setFocused(true);await f.tick(5000,4);await f.tick(6000,5);await f.tick(7000,6);
 assert.equal(f.plays().length,1);assert.ok(f.plays()[0].watched_ms>=2000);assert.equal(f.plays()[0].page_url,'https://www.youtube.com/shorts/Example123');
});
test('seeking to zero does not create a replay; a completed replay qualifies independently',async()=>{
 const f=await fixture();for(let t=1;t<=6;t++)await f.tick(t*1000,t-1);assert.equal(f.plays().length,1);
 f.playerEvents.seeking();await f.tick(7000,0);for(let t=8;t<=10;t++)await f.tick(t*1000,t-7);assert.equal(f.plays().length,1);
 f.playerEvents.ended();await f.tick(11000,0);await f.tick(12000,1);await f.tick(13000,2);assert.equal(f.plays().length,2);assert.notEqual(f.plays()[0].instance_id,f.plays()[1].instance_id);
});
test('playback speed and a suspended timer do not fabricate watched duration',async()=>{
 const f=await fixture('https://www.youtube.com/watch?v=Example123');await f.tick(1000,0,{playbackRate:2});await f.tick(2000,2);await f.tick(3000,4);assert.equal(f.plays()[0].watched_ms,2000);
 await f.tick(60000,60);const progress=f.messages.filter(m=>m.kind==='video.progress');assert.equal(progress.length,0);
});
test('Instagram DOM player replacement preserves the existing observed instance',async()=>{
 const f=await fixture('https://www.instagram.com/reel/Example123/');await f.tick(1000,0);await f.tick(2000,1);await f.tick(3000,2);const original=f.plays()[0].instance_id;
 const replacement={...f.player,dataset:{},currentTime:2};f.document.querySelectorAll=()=>[replacement];await f.tick(4000,3);replacement.currentTime=3;await f.tick(5000,4);assert.equal(f.plays().length,1);assert.equal(f.plays()[0].instance_id,original);
});
test('autoplay guard preserves deliberate playback and restores original player settings',async()=>{
 const f=await fixture();await f.policy([{domain:'www.youtube.com',action:'stop_autoplay',expires_at_ms:100000}]);assert.equal(f.player.autoplay,false);
 f.context.location=new URL('https://www.youtube.com/shorts/Another123');f.events.play({target:f.player});assert.equal(f.player.paused,true);
 f.events.pointerdown({isTrusted:true,type:'pointerdown',target:{closest:()=>f.player}});f.player.paused=false;f.events.play({target:f.player});assert.equal(f.player.paused,false);
 await f.policy([]);assert.equal(f.player.autoplay,true);assert.equal(f.player.attrs.autoplay,'');
});
