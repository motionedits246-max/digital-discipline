import test from 'node:test';import assert from 'node:assert/strict';
import {matchesRule,learningAllowRules} from '../extension/src/rules.js';
const tutorial='Example123',watch='https://www.youtube.com/watch?v='+tutorial;
const focus={id:'focus',domain:'www.youtube.com',action:'block',path_prefix:null,allow_video_ids:[tutorial]};
test('Learning Mode permits the selected direct tutorial and continues blocking other routes',()=>{
 assert.equal(matchesRule(watch,focus),false);assert.equal(matchesRule('https://www.youtube.com/watch?v=Other123',focus),true);assert.equal(matchesRule('https://www.youtube.com/shorts/'+tutorial,focus),true);
 assert.equal(matchesRule('https://www.youtube.com/watch?v=Other123&v='+tutorial,focus),true);
 const [allow]=learningAllowRules([focus],801000);assert.equal(allow.action.type,'allow');const re=new RegExp(allow.condition.regexFilter);assert.equal(re.test(watch),true);assert.equal(re.test('https://www.youtube.com/watch?v=Other123&v='+tutorial),false);
});
test('one Learning Mode exception never bypasses an unrelated hard rule',()=>{
 const independent={...focus,id:'independent',allow_video_ids:undefined};assert.deepEqual(learningAllowRules([focus,independent],801000),[]);
 assert.equal(learningAllowRules([focus,{domain:'www.instagram.com',action:'block'}],801000).length,1);
});
