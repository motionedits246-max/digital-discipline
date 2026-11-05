import type {ReactNode} from 'react';
const icons:Record<string,ReactNode>={
  overview:<><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></>,
  activity:<><path d="M3 12h4l3-7 4 14 3-7h4"/></>,
  history:<><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M8 2v4M16 2v4M3 10h18"/></>,
  video:<><rect x="3" y="4" width="18" height="16" rx="4"/><path d="m10 9 5 3-5 3z"/></>,
  target:<><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1"/></>,
  shield:<><path d="m12 2 8 4v6c0 5-4 8-8 10-4-2-8-5-8-10V6z"/><path d="m9 12 2 2 4-4"/></>,
  branch:<><circle cx="6" cy="3" r="2"/><circle cx="6" cy="21" r="2"/><circle cx="18" cy="6" r="2"/><path d="M6 5v14M6 12c0-4 12-1 12-6"/></>,
  insights:<><path d="M4 18V9M9 18V5M14 18v-7M19 18V3M2 21h20"/></>,
  sparkle:<><path d="m12 2 2.2 7.8L22 12l-7.8 2.2L12 22l-2.2-7.8L2 12l7.8-2.2z"/></>,
  settings:<><circle cx="12" cy="12" r="3"/><path d="M19 12a7 7 0 0 0-.15-1.4l2-1.6-2-3.5-2.5 1a7 7 0 0 0-2.4-1.4L13.5 2h-4L9 5.1a7 7 0 0 0-2.4 1.4l-2.5-1-2 3.5 2 1.6A7 7 0 0 0 4 12a7 7 0 0 0 .15 1.4l-2 1.6 2 3.5 2.5-1a7 7 0 0 0 2.4 1.4l.5 3.1h4l.5-3.1a7 7 0 0 0 2.4-1.4l2.5 1 2-3.5-2-1.6A7 7 0 0 0 19 12z"/></>,
  chevron:<><path d="m9 18 6-6-6-6"/></>, pause:<><path d="M8 5v14M16 5v14"/></>, play:<><path d="m8 5 11 7-11 7z"/></>,
  plus:<><path d="M12 5v14M5 12h14"/></>,close:<><path d="M5 5 19 19M19 5 5 19"/></>,
  expand:<><path d="M8 3H3v5M16 3h5v5M3 16v5h5M21 16v5h-5"/></>,
  window:<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18"/></>,
  maximize:<><rect x="4" y="4" width="16" height="16" rx="2"/></>,
  restore:<><rect x="7" y="3" width="13" height="13" rx="2"/><path d="M16 18v2H4V7h2"/></>,
  arrow:<><path d="M5 12h14m-6-6 6 6-6 6"/></>,download:<><path d="M12 3v12m-5-5 5 5 5-5M4 17v4h16v-4"/></>,
  clock:<><circle cx="12" cy="12" r="9"/><path d="M12 7v5l4 2"/></>,check:<><path d="m4 12 5 5L20 6"/></>,
  search:<><circle cx="10.5" cy="10.5" r="7"/><path d="m16 16 5 5"/></>, dot:<><circle cx="12" cy="12" r="3"/></>,
  external:<><path d="M13 4h7v7M20 4l-9 9"/><path d="M20 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h5"/></>,
};
export function Icon({name,size=18}:{name:string;size?:number}){
 return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{icons[name]||icons.dot}</svg>;
}
