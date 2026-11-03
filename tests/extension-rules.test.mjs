import test from 'node:test';
import assert from 'node:assert/strict';
import {isValidDomain,safePrefix,matchesRule,regexFor} from '../extension/src/rules.js';

test('valid target scope rejects scheme, usernames, IP and malformed domains',()=>{
 assert.equal(isValidDomain('youtube.com'),true);
 for(const d of ['https://youtube.com','youtube.com/abc','hello','-evil.com','youtube..com','youtube.com:443','127.0.0.1'])assert.equal(isValidDomain(d),false,d);
});
test('Shorts route does not block normal YouTube watch page',()=>{
 const r={domain:'www.youtube.com',path_prefix:'/shorts/'};
 assert.equal(matchesRule('https://www.youtube.com/shorts/ABC123',r),true);
 assert.equal(matchesRule('https://www.youtube.com/watch?v=ABC123',r),false);
 assert.equal(matchesRule('https://www.youtube.com.evil.net/shorts/ABC123',r),false);
 assert.equal(matchesRule('http://malicious.net/?url=www.youtube.com/shorts/',r),false);
});
test('DNR regex anchors host and route',()=>{
 const pattern=new RegExp(regexFor({domain:'www.youtube.com',path_prefix:'/shorts/'}));
 assert.ok(pattern.test('https://www.youtube.com/shorts/A1'));
 assert.ok(!pattern.test('https://www.youtube.com/watch?v=A1'));
 assert.ok(!pattern.test('https://www.youtube.com.evil.net/shorts/A1'));
});
test('unsafe path prefixes rejected',()=>{
 assert.equal(safePrefix('/shorts/'),true);
 for(const p of ['/watch?token=private','/../../','\\evil','/foo#bar'])assert.equal(safePrefix(p),false,p);
});

test('top-level domain rule cannot match a longer lookalike hostname',()=>{
 const pattern=new RegExp(regexFor({domain:'youtube.com',path_prefix:null}));
 for(const ok of ['https://youtube.com/','https://www.youtube.com/shorts/a','https://youtube.com:443/watch','http://youtube.com'])assert.equal(pattern.test(ok),true,ok);
 for(const bad of ['https://youtube.com.evil.org/','https://youtube.comevil.org/','https://otheryoutube.com/','https://example.org/?target=youtube.com'])assert.equal(pattern.test(bad),false,bad);
});
test('route-only rule accepts a specified path but not unrelated pages',()=>{
 const r={domain:'www.youtube.com',path_prefix:'/shorts/'};
 assert.equal(matchesRule('https://www.youtube.com/shorts/xyz',r),true);
 assert.equal(matchesRule('https://www.youtube.com/shorts',r),false);
 assert.equal(matchesRule('https://www.youtube.com/feed/subscriptions',r),false);
});
test('site matching excludes non-http schemes and lookalike hosts',()=>{
 for(const url of ['file:///www.youtube.com/shorts/a','ftp://www.youtube.com/shorts/a','https://youtube.com.evil.org/shorts/a'])assert.equal(matchesRule(url,{domain:'youtube.com',path_prefix:null}),false);
});
test('URL path patterns cannot inject regex syntax',()=>{
 for(const path of ['/(.*)','/a|b','/abc\\','/a?b','/a#b'])assert.equal(safePrefix(path),false,path);
});
