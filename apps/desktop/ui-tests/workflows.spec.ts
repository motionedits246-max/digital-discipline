import {test,expect,type Page} from '@playwright/test';
const rule='11111111-1111-4111-8111-111111111111',task='22222222-2222-4222-8222-222222222222',profile='33333333-3333-4333-8333-333333333333';
async function connected(page:Page,paired=true){
 await page.addInitScript(({rule,task,profile,paired})=>{
  localStorage.setItem('dd:guided-tour-complete-v1','true');
  const calls:any[]=[];let focus:any=null,privateState={active:false,until_ms:null as number|null,automatic_ai_paused:false,restrictions_continue:true};let exceptions:any[]=[];
  (window as any).__testCalls=calls;const callbacks=new Map<number,(event:any)=>void>(),nativeEvents:Record<string,(event:any)=>void>={};let callbackId=0;
  (window as any).__testNativeEvents=nativeEvents;
  (window as any).__TAURI_INTERNALS__={transformCallback:(fn:(event:any)=>void)=>{callbacks.set(++callbackId,fn);return callbackId;},unregisterCallback:()=>{},invoke:async(command:string,args:any)=>{
   calls.push({command,args});if(command==='plugin:event|listen'){nativeEvents[args.event]=callbacks.get(args.handler)!;return 1;}if(command!=='agent_request')return null;
   const {path,method,body}=args;
   if(path==='/state'||path==='/live')return {ok:true,schema_version:'1.0',observed_at_ms:Date.now(),tracking_paused:false,private_session:privateState,platform_supported:true,browser_connected:true,metrics:{active_ms:0,development_ms:0,entertainment_ms:0,qualified_plays:0,unique_videos:0,replays:0,playback_ms:0},current:null,sessions:[],heatmap:[],rules:[{id:rule,name:'No Shorts',domain:'www.youtube.com',path_prefix:'/shorts/',enabled:true,enforced:true}],goals:[],focus,profile_receipts:[],coverage:{windows_collector:'running',browser:'best_effort',no_data_is_not_zero:true}};
   if(path==='/browser-pairs')return {pairs:paired?[{browser:'Brave',extension_id:'a'.repeat(32)}]:[],pending:[]};
   if(path==='/workflows')return {profiles:[{id:profile,name:'Study',apps:['editor.exe'],domains:['www.youtube.com']}],tasks:[{id:task,profile_id:profile,title:'Finish exercises',state:'draft',outcome:''}]};
   if(path==='/focus/reviews')return {reviews:[]};if(path==='/tracking/doctor')return {profiles:[]};if(path==='/reports/heatmap')return {heatmap:[]};
   if(path==='/privacy/session'){if(method==='POST')privateState={active:body.minutes>0,until_ms:body.minutes?Date.now()+body.minutes*60000:null,automatic_ai_paused:body.minutes>0,restrictions_continue:true};return privateState;}
   if(path==='/policy/exceptions'){if(method==='POST')exceptions=[{id:'unlock',...body,rule_name:'No Shorts',expires_ms:Date.now()+body.minutes*60000}];return {ok:true,exceptions};}
   if(path==='/focus/start'){focus={id:'44444444-4444-4444-8444-444444444444',goal_id:null,profile_id:body.profile_id,task_id:body.task_id,start_ms:Date.now(),end_ms:Date.now()+3600000,difficulty:body.difficulty,block_domains:body.block_domains,context:{intended_outcome:'Finish exercises'},cycle:body.cycle?{state:'work',phase:'work',stage:0,cycles:body.cycle.cycles,ends_ms:Date.now()+1500000,remaining_ms:0,version:0}:undefined};return {ok:true,id:focus.id};}
   if(path.startsWith('/focus/cycles/')){focus.cycle={...focus.cycle,state:body.action==='pause'?'paused':'work',version:focus.cycle.version+1};return {ok:true};}
   if(path==='/settings')return {settings:{retention_days:90,idle_threshold_ms:300000,excluded_apps:[],excluded_domains:[],video_history_visible:true,distraction_categories:[],prediction_opt_in:false,coach_share_opt_in:false}};
   if(path==='/health/details')return {sampled_at_ms:Date.now(),agent:{reachable:true,tracking_paused:false},browser:{last_message_at_ms:0,last_rule_ack_at_ms:0,recent_signal:false,recent_rule_ack:false},storage:{db_bytes:1000,persisted_event_count:0,session_rows:0,last_accepted_event_at_ms:null},optional_workers:{auto_classification_enabled:false},redacted:true};
   if(path==='/categories')return {overrides:[]};if(path==='/classifications')return {jobs:[],enabled:false,key_saved:false,memory:{remembered_services:0,requests_today:0,daily_request_limit:20,automatic_review:true}};
   if(path==='/coach/credentials')return {providers:[]};return {ok:true};
  }};
  (window as any).__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
 },{rule,task,profile,paired});
 await page.goto('/');if(!paired){await expect(page.getByRole('heading',{name:'Connect your browser',exact:true})).toBeVisible();return;}await page.getByRole('button',{name:'Open quick panel'}).click();await page.getByRole('button',{name:'Open workspace',exact:false}).click();
}
test('native tray privacy masks browser setup and preserves its extension draft',async({page})=>{
 await connected(page,false);await page.getByRole('button',{name:/^Go to step 3:/}).click();const draft=page.getByLabel('Extension ID',{exact:true});await draft.fill('a'.repeat(32));
 await page.waitForFunction(()=>!!(window as any).__testNativeEvents['dd:toggle-privacy']);
 await page.evaluate(()=>(window as any).__testNativeEvents['dd:toggle-privacy']({event:'dd:toggle-privacy',id:1,payload:null}));
 await expect(page.getByRole('heading',{name:'Your activity details are hidden'})).toBeVisible();await expect(draft).not.toBeVisible();
 await page.getByRole('button',{name:'Show my workspace',exact:true}).click();await expect(draft).toHaveValue('a'.repeat(32));
});
test('saved workflow submits task, cycles and selected tutorial scope, then supports pause',async({page})=>{
 await connected(page);await page.getByRole('button',{name:'Focus & Goals',exact:true}).click();
 const section=page.getByRole('region',{name:'Focus workflow and reviews'});
 await section.getByText('No attached task',{exact:true}).click();await page.getByRole('option',{name:'Finish exercises',exact:true}).click();
 await section.getByText('One timebox',{exact:true}).click();await page.getByRole('option',{name:'Work and break cycles',exact:true}).click();
 await section.getByRole('button',{name:'Enable Learning Mode',exact:true}).click();await section.getByLabel('Allowed tutorial links or video IDs').fill('https://www.youtube.com/watch?v=Example123');
 await section.getByRole('button',{name:'Start saved focus workflow'}).click();await expect(section.getByRole('button',{name:'Pause cycle',exact:true})).toBeVisible();
 const body=await page.evaluate(()=>(window as any).__testCalls.find((c:any)=>c.args?.path==='/focus/start').args.body);expect(body.task_id).toBe(task);expect(body.profile_id).toBe(profile);expect(body.cycle.work_minutes).toBe(25);expect(body.learning.video_ids).toEqual(['Example123']);expect(body.learning.override_rule_ids).toEqual(['focus']);
 await section.getByRole('button',{name:'Pause cycle',exact:true}).click();await expect(section.getByRole('button',{name:'Resume cycle',exact:true})).toBeVisible();
});
test('private-session controls preserve a declared duration and provide an end action',async({page})=>{
 await connected(page);await page.getByRole('button',{name:'Settings & Privacy',exact:true}).click();const panel=page.getByRole('article',{name:'Private sessions and scoped deletion'});await panel.getByRole('button',{name:'Start private session'}).click();await expect(panel.getByRole('button',{name:'End private session'})).toBeVisible();await panel.getByRole('button',{name:'End private session'}).click();await expect(panel.getByRole('button',{name:'Start private session'})).toBeVisible();
 const bodies=await page.evaluate(()=>(window as any).__testCalls.filter((c:any)=>c.args?.path==='/privacy/session'&&c.args?.method==='POST').map((c:any)=>c.args.body.minutes));expect(bodies).toEqual([30,0]);
});
test('normal unlock sends a selected rule and site rather than emergency release',async({page})=>{
 await connected(page);await page.getByRole('button',{name:'Blocking Center',exact:true}).click();const panel=page.getByRole('article',{name:'Scoped temporary unlock'});await panel.getByText('Choose a restriction',{exact:true}).click();await page.getByRole('option',{name:'No Shorts · www.youtube.com',exact:true}).click();await panel.getByRole('button',{name:'Create scoped unlock'}).click();await expect(panel.getByRole('button',{name:'End unlock'})).toBeVisible();const body=await page.evaluate(()=>(window as any).__testCalls.find((c:any)=>c.args?.path==='/policy/exceptions'&&c.args?.method==='POST').args.body);expect(body).toEqual({rule_id:rule,domain:'www.youtube.com',minutes:5,reason:'planned_use',profile_id:null});
});
