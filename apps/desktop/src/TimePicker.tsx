type Props={value:string;onChange:(e:{target:{value:string}})=>void;'aria-label'?:string};
export function TimePicker({value,onChange,'aria-label':label='Time'}:Props){
 const emit=(next:string)=>onChange({target:{value:next}});
 const step=(delta:number)=>{const parts=value.match(/^(\d{1,2}):(\d{2})$/);const minutes=parts?+parts[1]*60+(+parts[2]):0;const next=(minutes+delta+1440)%1440;emit(String(Math.floor(next/60)).padStart(2,'0')+':'+String(next%60).padStart(2,'0'));};
 return <span className="dd-time"><input type="text" inputMode="numeric" maxLength={5} pattern="([01][0-9]|2[0-3]):[0-5][0-9]" placeholder="HH:MM" aria-label={label} autoComplete="off" value={value} onChange={e=>emit(e.target.value)} onBlur={()=>{const match=value.match(/^(\d{1,2}):(\d{1,2})$/);if(match&&+match[1]<24&&+match[2]<60)emit(match[1].padStart(2,'0')+':'+match[2].padStart(2,'0'));}}/>
  <button type="button" aria-label={'Earlier '+label} title="Five minutes earlier" onClick={()=>step(-5)}>−</button><button type="button" aria-label={'Later '+label} title="Five minutes later" onClick={()=>step(5)}>+</button>{value&&<button type="button" aria-label={'Clear '+label} onClick={()=>emit('')}>×</button>}</span>;
}
