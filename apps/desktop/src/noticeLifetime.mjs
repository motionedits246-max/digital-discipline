// The exit animation is included in the three-second visibility limit.
export const NOTICE_DURATION_MS=3000;
export const NOTICE_EXIT_MS=170;
export function createNoticeLifetime({onShow=()=>{},onExit,onHide,setTimer=setTimeout,clearTimer=clearTimeout}){
 let expiry,hide,state='hidden';
 const clear=()=>{clearTimer(expiry);clearTimer(hide);expiry=hide=undefined;};
 const exit=()=>{if(state==='hidden'||state==='leaving')return;clear();state='leaving';onExit();hide=setTimer(()=>{state='hidden';onHide();},NOTICE_EXIT_MS);};
 const arm=()=>{expiry=setTimer(exit,NOTICE_DURATION_MS-NOTICE_EXIT_MS);};
 return {
  show(){clear();state='transient';onShow();arm();},
  interact(){if(state==='hidden')return;clear();state='engaged';onShow();},
  away(){if(state!=='engaged')return false;clear();state='transient';arm();return true;},
  dismiss:exit,
  dispose(){clear();state='hidden';},
 };
}
