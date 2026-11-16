// Shared pure telemetry rules, loaded before the isolated content adapter.
(()=>{
 function observedDelta({wall,delta,rate=1,visible,paused,seeking,ready}){
  if(!visible||paused||seeking||ready<2||!Number.isFinite(wall)||!Number.isFinite(delta)||wall<=0||wall>2500||delta<=0||!Number.isFinite(rate)||rate<0.1||rate>16)return 0;
  const progress=delta*1000/rate;
  // Seeking, stalled media and timer suspension do not become watched time.
  if(progress>wall*1.75+250)return 0;
  return Math.min(wall,progress,1200);
 }
 function replayConfirmed({ended,loop,previous,media,duration,seeking,wall,manual}){
  if(media<0||media>1.5||wall<0||wall>2500)return false;
  if(ended&&(!seeking||manual))return true;
  return !seeking&&loop&&Number.isFinite(duration)&&duration>2&&previous>=duration-1.5&&media<previous-2;
 }
 globalThis.DDPlayback=Object.freeze({observedDelta,replayConfirmed});
})();
