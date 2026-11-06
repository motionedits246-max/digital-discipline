import {useEffect,useState} from 'react';
import {agent} from './api';

type ClassifierStatus={enabled:boolean;provider:string;model:string;key_saved:boolean;external_requests_automatic:boolean;batch_size:number;
 last_attempt_at_ms?:number|null;last_result?:string;last_error_code?:string};
type PrivacyStatus={classifier:ClassifierStatus;coachConsent:boolean;savedCoachProviders:string[];verifiedAt:number};
const outcomes:Record<string,string>={in_progress:'Request in progress',succeeded:'Response received',failed:'Request failed',
 interrupted:'Interrupted; recovery pending',disabled_after_send:'Disabled after transmission'};
const failures:Record<string,string>={invalid_key:'The provider rejected the API key.',provider_access_denied:'The provider denied access.',
 model_unavailable:'The selected model is unavailable.',rate_limited:'The provider rate limit was reached.',
 provider_unavailable:'The provider is unavailable.',worker_interrupted:'The previous request was interrupted.'};

export function NetworkPrivacyStatus({readonly}:{readonly:boolean}){
 const [status,setStatus]=useState<PrivacyStatus|null>(null),[error,setError]=useState('');
 useEffect(()=>{
  if(readonly)return;
  let mounted=true,running=false;
  const refresh=async()=>{
   if(running)return;running=true;
   try{
    const [classifier,keys,settings]=await Promise.all([
     agent<ClassifierStatus>('GET','/classifications'),
     agent<{providers:string[]}>('GET','/coach/credentials'),
     agent<{settings:{coach_share_opt_in:boolean}}>('GET','/settings')
    ]);
    if(mounted){setStatus({classifier,coachConsent:settings.settings.coach_share_opt_in===true,savedCoachProviders:keys.providers,verifiedAt:Date.now()});setError('');}
   }catch{if(mounted)setError('Could not verify current privacy status.');}
   finally{running=false;}
  };
  void refresh();const timer=window.setInterval(()=>void refresh(),15000);
  return()=>{mounted=false;window.clearInterval(timer);};
 },[readonly]);
 const verified=!readonly&&!error&&status!==null;
 const classifier=status?.classifier;
 const automatic=verified&&classifier?.external_requests_automatic;
 const label=!verified?'Unknown':automatic?'On':classifier?.enabled?'Needs attention':'Off';
 const latest=classifier?.last_attempt_at_ms;
 return <div aria-label="AI privacy status" aria-live="polite">
  <div className="settings-row"><div><strong>Automatic external classification</strong>
   <small>{readonly?'Status unavailable while the agent is offline or in sample preview.':error||(!status?'Checking local agent…':automatic?
    'Enabled · '+classifier?.provider+' · '+classifier?.model:
    classifier?.enabled?'Enabled, but a saved key or valid provider configuration is missing.':'Disabled · no new automatic classification requests')}</small>
   {verified&&<small>Classifier key: {classifier?.key_saved?'encrypted file present; checked when used':'not saved'}. {classifier?.provider?classifier.provider+' / '+classifier.model:''}</small>}
   {verified&&<small>{latest?'Last request: '+new Date(latest).toLocaleString()+' · '+(outcomes[classifier?.last_result||'']||'Result unavailable'):'No automatic request recorded yet.'}
    {classifier?.last_error_code?' '+(failures[classifier.last_error_code]||'The request could not complete; review classification settings.'):''}</small>}
   <small>Only public hostnames and application basenames may be sent, up to {classifier?.batch_size||10} names per request. No full URLs, document titles or executable paths. Disabling prevents future requests; data already sent cannot be recalled.</small>
  </div><span className={'tag '+(automatic?'tag-on':'')}>{label}</span></div>
  <div className="settings-row"><div><strong>AI Coach sharing and saved keys</strong>
   <small>{verified?(status.coachConsent?'Aggregate sharing preference enabled. Each chat request is initiated by you.':'Aggregate sharing preference disabled.'):
    'Current Coach consent and key status cannot be verified.'}</small>
   <small>{verified?(status.savedCoachProviders.length?'Encrypted Coach keys: '+status.savedCoachProviders.join(', '):'No encrypted Coach key files saved.'):''}</small>
   <small>Coach sharing is separate from automatic classification. One-time keys are used for one request; saved keys are protected for the current Windows user and are never displayed here.</small>
  </div><span className="tag">{!verified?'Unknown':status.coachConsent?'Sharing allowed':'Sharing off'}</span></div>
  {status&&<small className="muted">Last verified locally: {new Date(status.verifiedAt).toLocaleTimeString()}</small>}
 </div>;
}
