/** User opt-in and local quiet hours, independent of tracking consent. */
export function validTime(value){return typeof value==='string' && /^([01]\d|2[0-3]):[0-5]\d$/.test(value);}
export function quietHours(now,start,end){
 if(!validTime(start)||!validTime(end)||start===end)return false;
 const mins=now.getHours()*60+now.getMinutes();
 const read=v=>Number(v.slice(0,2))*60+Number(v.slice(3,5));
 const a=read(start),b=read(end);
 return a<b ? mins>=a&&mins<b : mins>=a||mins<b;
}
export function shouldNotify(now,settings,previousMs){
 return settings.notificationsEnabled===true && settings.dndEnabled!==true &&
  !quietHours(now,settings.quietStart,settings.quietEnd) && now.getTime()-(previousMs||0)>=3600000;
}
