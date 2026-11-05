import {useEffect,useState} from 'react';
import {agent} from './api';

type Status={enabled:boolean;provider:string;model:string;key_saved:boolean;external_requests_automatic:boolean;batch_size:number};
export function NetworkPrivacyStatus({readonly}:{readonly:boolean}){
 const [status,setStatus]=useState<Status|null>(null),[error,setError]=useState('');
 useEffect(()=>{
  if(readonly)return;
  let mounted=true;
  const refresh=()=>void agent<Status>('GET','/classifications').then(x=>{if(mounted){setStatus(x);setError('');}}).catch(()=>{if(mounted)setError('Could not verify background AI status.');});
  refresh();const timer=window.setInterval(refresh,15000);
  return()=>{mounted=false;window.clearInterval(timer);};
 },[readonly]);
 return <div className="settings-row"><div><strong>Automatic external classification</strong>
  <small>{readonly?'Unavailable in demo mode':error||(!status?'Checking local agent…':status.external_requests_automatic?
   `Enabled · ${status.provider} · ${status.model} · up to ${status.batch_size} public names/request. Optional outbound AI calls may occur.`:
   status.enabled?'Enabled but unavailable: no usable saved key or provider configuration.':'Disabled · no automatic AI classification requests')}</small>
  <small>Only public hostnames and application basenames may be transmitted with your consent. No full browsing URLs, document titles or executable paths.</small>
  <small>AI Coach is a separate optional capability. API keys can be saved encrypted for the Windows user or supplied once; they are not shown here.</small>
 </div><span className={'tag '+(status?.external_requests_automatic?'tag-on':'')}>{status?.external_requests_automatic?'On':status?.enabled?'Needs attention':'Off'}</span></div>;
}
