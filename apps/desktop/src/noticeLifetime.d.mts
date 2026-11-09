export const NOTICE_DURATION_MS:number;
export const NOTICE_EXIT_MS:number;
export function createNoticeLifetime(options:{onShow?:()=>void;onExit:()=>void;onHide:()=>void}):{show:()=>void;interact:()=>void;away:()=>boolean;dismiss:()=>void;dispose:()=>void};
