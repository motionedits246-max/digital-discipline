import type {BrowserPair,PendingBrowser} from './api';
import type {DesktopNotice} from './ServiceHint';
export function createNoticeFeed(startedAt?:number):{
 next(state:{pairs:BrowserPair[];pending:PendingBrowser[]},items?:{id:string;kind:string;title:string;created_ms:number;dismissed_ms:number|null}[]):{key:string;notice:DesktopNotice}|null;
 acknowledge:(key:string)=>void;
};
