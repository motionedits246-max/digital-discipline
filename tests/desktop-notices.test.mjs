import test from 'node:test';
import assert from 'node:assert/strict';
import {createNoticeLifetime} from '../apps/desktop/src/noticeLifetime.mjs';
import {createNoticeFeed} from '../apps/desktop/src/noticeFeed.mjs';

function clock(){
 let now=0,id=0;const timers=new Map(),events=[];
 const life=createNoticeLifetime({setTimer:(fn,delay)=>{timers.set(++id,{at:now+delay,fn});return id;},clearTimer:id=>timers.delete(id),onExit:()=>events.push(['exit',now]),onHide:()=>events.push(['hide',now])});
 return {life,events,tick(ms){const end=now+ms;for(;;){const next=[...timers].filter(([,v])=>v.at<=end).sort((a,b)=>a[1].at-b[1].at)[0];if(!next)break;now=next[1].at;timers.delete(next[0]);next[1].fn();}now=end;}};
}
test('unattended notification finishes its exit animation within three seconds',()=>{
 const c=clock();c.life.show();c.tick(2829);assert.deepEqual(c.events,[]);c.tick(1);assert.deepEqual(c.events,[['exit',2830]]);c.tick(170);assert.deepEqual(c.events.at(-1),['hide',3000]);
});
test('interaction holds a notification; repeated blur cannot extend its outside deadline',()=>{
 const c=clock();c.life.show();c.tick(2000);c.life.interact();c.tick(60000);assert.deepEqual(c.events,[]);assert.equal(c.life.away(),true);c.tick(2000);assert.equal(c.life.away(),false);c.tick(1000);assert.deepEqual(c.events.at(-1),['hide',65000]);
});
test('returning during the exit animation cancels the stale close',()=>{
 const c=clock();c.life.show();c.tick(2900);c.life.interact();c.tick(10000);assert.equal(c.events.some(e=>e[0]==='hide'),false);c.life.away();c.tick(3000);assert.equal(c.events.filter(e=>e[0]==='hide').length,1);
});
test('a new notification and disposed surfaces cannot inherit old hide callbacks',()=>{
 const c=clock();c.life.show();c.tick(2900);c.life.show();c.tick(200);assert.equal(c.events.some(e=>e[0]==='hide'),false);c.life.dispose();c.tick(10000);assert.equal(c.events.some(e=>e[0]==='hide'),false);
});
test('companion requests are announced once without authorizing them',()=>{
 const feed=createNoticeFeed(1000),state={pairs:[],pending:[{extension_id:'a'.repeat(32),requested_ms:1000}]};
 const item=feed.next(state);assert.equal(item.notice.title,'New browser connection');assert.equal(item.notice.page,'Settings & Privacy');assert.deepEqual(state.pairs,[]);
 assert.equal(feed.next(state).key,item.key);feed.acknowledge(item.key);assert.equal(feed.next({...state,pending:[{...state.pending[0],requested_ms:2000}]}),null);
 assert.equal(feed.next({pairs:[],pending:[]}),null);assert.equal(feed.next(state).notice.title,'New browser connection');
});
test('existing authorizations are quiet and a new approval stays distinct from recording consent',()=>{
 const feed=createNoticeFeed(1000),pair={browser:'Brave',extension_id:'a'.repeat(32),created_ms:100};assert.equal(feed.next({pairs:[pair],pending:[]}),null);
 const item=feed.next({pairs:[pair,{...pair,extension_id:'b'.repeat(32),created_ms:2000}],pending:[]});assert.equal(item.notice.title,'Browser authorized');assert.match(item.notice.detail,/requires your opt-in/);feed.acknowledge(item.key);assert.equal(feed.next({pairs:[pair,{...pair,extension_id:'b'.repeat(32),created_ms:2000}],pending:[]}),null);
});
test('old, dismissed and previously announced grouped inbox items do not create repeated overlays',()=>{
 const feed=createNoticeFeed(1000),state={pairs:[],pending:[]},item={id:'focus',kind:'focus',title:'Focus timebox ended',created_ms:1100,dismissed_ms:null};
 assert.equal(feed.next(state,[{...item,created_ms:100},{...item,id:'dismissed',dismissed_ms:1200}]),null);
 const notice=feed.next(state,[item]);assert.equal(notice.notice.page,'Inbox');feed.acknowledge(notice.key);assert.equal(feed.next(state,[{...item,occurrences:2,updated_ms:5000}]),null);
});
