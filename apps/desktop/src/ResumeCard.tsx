import {useEffect,useState} from 'react';import {agent} from './api';
type Bookmark={id:string;next_step:string;owner_outcome:string;dismissed:boolean;context:{intended_outcome?:string}};
export function ResumeCard({readonly,onOpen}:{readonly:boolean;onOpen:()=>void}){
 const [bookmark,setBookmark]=useState<Bookmark|null>(null);
 useEffect(()=>{if(readonly)return;let alive=true;void agent<{reviews:Bookmark[]}>('GET','/focus/reviews').then(r=>{if(alive)setBookmark(r.reviews.find(r=>!r.dismissed&&!!r.next_step)||null);}).catch(()=>{});return()=>{alive=false;};},[readonly]);
 if(!bookmark||readonly)return null;
 return <div className="resume-card"><small>YOUR NEXT STEP</small><p>{bookmark.next_step}</p><button className="secondary-btn" onClick={onOpen}>Resume task</button><button className="icon-btn" aria-label="Dismiss resume card" onClick={()=>{void agent('POST',`/focus/reviews/${bookmark.id}`,{outcome:bookmark.owner_outcome,next_step:bookmark.next_step,dismissed:true}).then(()=>setBookmark(null)).catch(()=>{});}}>×</button></div>;
}
