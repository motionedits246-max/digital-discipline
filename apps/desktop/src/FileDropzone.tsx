import {useRef,useState} from 'react';
export function FileDropzone({onFile,disabled=false}:{onFile:(file:File|undefined)=>void|Promise<void>;disabled?:boolean}){
 const input=useRef<HTMLInputElement>(null);const [name,setName]=useState('');const [dragging,setDragging]=useState(false);
 function accept(file:File|undefined){if(!file)return;setName(file.name);void onFile(file);}
 return <div className="dd-file-wrap">
   <span className="dd-upload-title">Import encrypted file</span>
   <button type="button" className={'dd-upload'+(dragging?' dragging':'')} disabled={disabled} onClick={()=>input.current?.click()} onDragOver={e=>{e.preventDefault();setDragging(true);}} onDragLeave={e=>{e.preventDefault();setDragging(false);}} onDrop={e=>{e.preventDefault();setDragging(false);if(!disabled)accept(e.dataTransfer.files[0]);}} aria-label="Choose an encrypted device bundle file">
     <span className="dd-upload-mark" aria-hidden="true">↑</span>
     <span><strong>{name||'Choose a bundle file'}</strong><small>{name?'Ready for decryption with your passphrase':'Click to browse or drag a .json file here · 650 KB max'}</small></span>
   </button>
   <input ref={input} type="file" accept=".json,application/json,text/plain" tabIndex={-1} className="dd-file-hidden" onChange={e=>{accept(e.target.files?.[0]);e.target.value='';}} aria-label="Browse for encrypted JSON file"/>
 </div>;
}
