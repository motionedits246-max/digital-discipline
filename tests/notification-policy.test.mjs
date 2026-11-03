import {test} from 'node:test';import {strict as assert} from 'node:assert';
import {quietHours,shouldNotify,validTime} from '../extension/src/notifications.js';
test('rejects malformed times',()=>{assert.equal(validTime('25:70'),false);assert.equal(validTime('22:15'),true);});
test('overnight quiet hours suppress 23:30 and 03:00, not 12:00',()=>{
 const mk=(h,m)=>new Date(2026,0,1,h,m);
 assert.equal(quietHours(mk(23,30),'22:00','07:00'),true);
 assert.equal(quietHours(mk(3,0),'22:00','07:00'),true);
 assert.equal(quietHours(mk(12,0),'22:00','07:00'),false);
});
test('DND, consent, cooldown, and quiet hours all respected',()=>{
 const now=new Date(2026,0,1,12,0),state={notificationsEnabled:true,dndEnabled:false,quietStart:'22:00',quietEnd:'07:00'};
 assert.equal(shouldNotify(now,state,0),true);
 assert.equal(shouldNotify(now,state,now.getTime()-1000),false);
 assert.equal(shouldNotify(now,{...state,dndEnabled:true},0),false);
 assert.equal(shouldNotify(now,{...state,notificationsEnabled:false},0),false);
});
