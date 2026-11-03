import type {Snapshot,Session} from './types';
// Explicitly labeled design-preview fixture, never included in real reports.
export function makeDemo():Snapshot {
 const now=Date.now();const date=new Date();const dayStart=new Date(date.getFullYear(),date.getMonth(),date.getDate()).getTime();
 const apps:[string,string,number,number,string|null][]=[
  ['cursor.exe','Development',7.5,72,null],['brave.exe','Development',9.5,34,'github.com'],
  ['brave.exe','Entertainment',10.25,25,'www.youtube.com'],['antigravity.exe','Development',11,101,null],
  ['explorer.exe','Utility',13.15,14,null],['brave.exe','Entertainment',13.7,37,'www.instagram.com'],
  ['pycharm64.exe','Development',14.5,66,null],['brave.exe','Entertainment',16,19,'www.youtube.com']
 ];
 const sessions:Session[]=apps.map(([app,category,hour,minutes,domain],i)=>({id:String(i),app,category,start_ms:dayStart+hour*3600_000,end_ms:dayStart+(hour*60+minutes)*60_000,idle:false,domain})).filter(s=>s.start_ms<=now).map(s=>({...s,end_ms:Math.min(s.end_ms,now)})).reverse();
 const heatmap=Array.from({length:365},(_,i)=>{
  const d=new Date(dayStart-(364-i)*86_400_000);const week=d.getDay();
  const active=i<364? (week===0 ? 0:((i*19+Math.floor(i/7)*3)%245)*60_000) : 260*60_000;
  return {date:[d.getFullYear(),String(d.getMonth()+1).padStart(2,'0'),String(d.getDate()).padStart(2,'0')].join('-'),active_ms:active,development_ms:active*.72,entertainment_ms:active*.19,observed:i>10};
 });
 return {ok:true,schema_version:'1.0',observed_at_ms:now,tracking_paused:false,platform_supported:true,browser_connected:true,
  metrics:{active_ms:360*60_000,development_ms:252*60_000,entertainment_ms:81*60_000,qualified_plays:23,unique_videos:18,replays:5,playback_ms:47*60_000},
  current:sessions[0]||null,sessions,heatmap,rules:[{id:'preview-rule',name:'Shorts cooldown',domain:'www.youtube.com',path_prefix:'/shorts/',expires_at_ms:now+50*60_000,budget_minutes:null,enabled:true,enforced:true}],
  coverage:{windows_collector:'sample',browser:'sample',no_data_is_not_zero:true}};
}
