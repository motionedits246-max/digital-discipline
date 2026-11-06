import type {Snapshot} from './types';
export const isTauri = !!(window as Window & {__TAURI_INTERNALS__?:unknown}).__TAURI_INTERNALS__;
async function invoke<T>(name:string,args:Record<string,unknown>):Promise<T> {
  const {invoke} = await import('@tauri-apps/api/core');
  return invoke<T>(name,args);
}
export async function agent<T=unknown>(method:'GET'|'POST'|'DELETE',path:string,body?:unknown):Promise<T> {
  if (!isTauri) throw new Error('Browser preview has no privileged agent access');
  return invoke<T>('agent_request',{method,path,body:body??null});
}
export async function snapshot(compact=false):Promise<Snapshot> { return agent<Snapshot>('GET',compact?'/live':'/state'); }
export async function presentation(mode:'pill'|'quick'|'full'|'popout'):Promise<void> {
  if(isTauri) await invoke<void>('set_presentation',{mode});
}

export async function dragWindow():Promise<void>{
  if(isTauri)await invoke<void>('drag_window',{});
}
export async function hideToTray():Promise<void>{
  if(isTauri)await invoke<void>('hide_to_tray',{});
}
export type BrowserPair={browser:'Brave'|'Chrome'|'Edge';extension_id:string;created_ms:number;last_seen_ms:number};
export type PendingBrowser={extension_id:string;requested_ms:number};
export async function getBrowserPairState():Promise<{pairs:BrowserPair[];pending:PendingBrowser[]}>{
  return agent<{pairs:BrowserPair[];pending:PendingBrowser[]}>('GET','/browser-pairs');
}
export async function pairBrowser(browser:'Brave'|'Chrome'|'Edge',extensionId:string):Promise<string>{
  const r=await agent<{message:string}>('POST','/browser-pairs',{browser,extension_id:extensionId});
  return r.message;
}
export async function forgetBrowserPair(extensionId:string):Promise<void>{
  await agent('DELETE','/browser-pairs/'+extensionId);
}

export async function autostartStatus():Promise<boolean>{
  if(!isTauri)return false;
  return invoke<boolean>('autostart_status',{});
}
export async function setAutostart(enabled:boolean):Promise<string>{
  if(!isTauri)throw new Error('Requires Windows desktop app');
  return invoke<string>('set_autostart',{enabled});
}

export type BrowserDiagnostics={manifest_exists:boolean;manifest_valid:boolean;registry_matches:boolean;origin_allowed:boolean;host_exists:boolean;agent_reachable:boolean;extension_bundled:boolean;manifest_path:string;host_path:string;note:string};
export async function openExtensionFolder():Promise<string>{return invoke<string>('open_extension_folder',{});}
export async function browserDiagnostics(browser:'Brave'|'Chrome'|'Edge',extensionId:string):Promise<BrowserDiagnostics>{return invoke<BrowserDiagnostics>('browser_pair_diagnostics',{browser,extensionId});}

export type LifecycleStatus={state:string;restart_count?:number;expected_version?:string;wanted?:boolean};
export async function lifecycleStatus():Promise<LifecycleStatus>{return isTauri?invoke<LifecycleStatus>('lifecycle_status',{}):{state:'preview'};}
