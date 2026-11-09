// App-local companion events only. Website account sign-ins are not collected.
export function createNoticeFeed(startedAt=Date.now()){
 let initialized=false;const seen=new Set();
 const pairKey=p=>'pair:'+p.extension_id+':'+p.created_ms;
 return {
  next(state,items=[]){
   if(!initialized){for(const pair of state.pairs)seen.add(pairKey(pair));initialized=true;}
   const activeRequests=new Set(state.pending.map(p=>'pending:'+p.extension_id));
   for(const key of seen)if(key.startsWith('pending:')&&!activeRequests.has(key))seen.delete(key);
   for(const pending of state.pending){
    const key='pending:'+pending.extension_id;
    if(!seen.has(key))return {key,notice:{kind:'browser',title:'New browser connection',detail:'A browser companion is asking to connect. Review its Extension ID before authorizing it.',page:'Settings & Privacy'}};
   }
   for(const pair of state.pairs){
    const key=pairKey(pair);
    if(!seen.has(key))return {key,notice:{kind:'browser',title:'Browser authorized',detail:pair.browser+' companion is approved. Browser recording still requires your opt-in.',page:'Settings & Privacy'}};
   }
   for(const item of items){
    const key='inbox:'+item.id;
    if(!item.dismissed_ms&&item.created_ms>=startedAt&&!seen.has(key))return {key,notice:{kind:'status',title:item.title,detail:item.kind==='focus'?'Your timebox ended. Review your session and choose your next step.':'Open the status inbox to review this update.',page:'Inbox'}};
   }
   return null;
  },
  acknowledge(key){seen.add(key);},
 };
}
