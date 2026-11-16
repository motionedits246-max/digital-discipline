// A forced refresh after a mutation invalidates the older result. Polls coalesce.
export function createRefreshCoordinator(load,apply,onError){
 let running=null,again=false,disposed=false;
 return {
  refresh(force=false){
   if(disposed)return Promise.resolve();
   if(running){again=again||force;return running;}
   running=(async()=>{do{again=false;try{const value=await load();if(!again&&!disposed)apply(value);}catch(error){if(!again&&!disposed)onError(error);}}while(again&&!disposed);})().finally(()=>{running=null;});
   return running;
  },
  dispose(){disposed=true;again=false;}
 };
}
