import {Icon} from './icons';
export function PresentationShield({paused,privateActive=false,offline,onReveal,onPause}:{paused:boolean;privateActive?:boolean;offline:boolean;onReveal:()=>void;onPause:()=>void}){
 return <article className="presentation-shield" aria-label="Presentation mode is active">
  <div className="presentation-symbol"><Icon name="shield" size={34}/></div><span className="eyebrow">PRESENTATION MODE</span><h1>Your activity details are hidden</h1>
  <p>Share your screen with a clear workspace. App names, sites, charts and conversations stay out of view.</p>
  <div className="presentation-state"><span className={'health-dot '+(offline?'':'on')}/>{offline?'Agent connection unavailable':privateActive?'Private session is active':paused?'Tracking is paused':'Tracking continues in the background'}<span>Browser restrictions keep their own settings.</span></div>
  <div className="device-actions"><button type="button" className="primary-btn" onClick={onReveal}>Show my workspace</button><button type="button" className="secondary-btn" disabled={offline||privateActive} onClick={onPause}>{privateActive?'Private session active':paused?'Resume tracking':'Pause tracking'}</button></div>
 </article>;
}
