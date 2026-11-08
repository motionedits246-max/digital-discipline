//! Update only the owner's existing unpacked companion files. Never reinstall or touch browser storage.
use std::{fs,path::{Path,PathBuf},sync::{Arc,Mutex}};
use axum::{extract::State,http::StatusCode,Json};use serde::Deserialize;use serde_json::{json,Value};use rusqlite::params;
use crate::{AppState,ApiResult,err,features};
static UPDATE_LOCK:Mutex<()>=Mutex::new(());
pub const FILES:&[(&str,&[u8])]=&[
 ("blocked.html",include_bytes!("../../extension/blocked.html")),("blocked.js",include_bytes!("../../extension/blocked.js")),
 ("popup.html",include_bytes!("../../extension/popup.html")),("popup.js",include_bytes!("../../extension/popup.js")),("popup.css",include_bytes!("../../extension/popup.css")),
 ("icons/icon16.png",include_bytes!("../../extension/icons/icon16.png")),("icons/icon32.png",include_bytes!("../../extension/icons/icon32.png")),("icons/icon48.png",include_bytes!("../../extension/icons/icon48.png")),("icons/icon128.png",include_bytes!("../../extension/icons/icon128.png")),
 ("src/background.js",include_bytes!("../../extension/src/background.js")),("src/notifications.js",include_bytes!("../../extension/src/notifications.js")),("src/rules.js",include_bytes!("../../extension/src/rules.js")),
 ("src/playback-model.js",include_bytes!("../../extension/src/playback-model.js")),("src/queue-policy.js",include_bytes!("../../extension/src/queue-policy.js")),("src/video.js",include_bytes!("../../extension/src/video.js")),
 // Manifest is committed last; a browser reload is requested only after success.
 ("manifest.json",include_bytes!("../../extension/manifest.json"))];
pub fn version()->String{serde_json::from_slice::<Value>(FILES.last().unwrap().1).unwrap()["version"].as_str().unwrap().to_string()}
fn safe_path(path:&Path)->Result<(),String>{
 for p in path.ancestors(){if let Ok(meta)=fs::symlink_metadata(p){
  #[cfg(windows)] {use std::os::windows::fs::MetadataExt;if meta.file_attributes()&0x400!=0{return Err("Linked extension folders are unsupported".into());}}
  if meta.file_type().is_symlink(){return Err("Linked extension folders are unsupported".into());}
 }}Ok(())
}
fn recognized(root:&Path)->bool{fs::read(root.join("manifest.json")).ok().and_then(|d|serde_json::from_slice::<Value>(&d).ok()).is_some_and(|m|m["manifest_version"]==3&&m["name"].as_str().is_some_and(|n|n.starts_with("Digital Discipline"))&&m["background"]["service_worker"]=="src/background.js")}
fn contents(root:&Path,file:&str,bytes:&[u8])->Vec<u8>{
 if file!="manifest.json"{return bytes.to_vec();}
 let old=fs::read(root.join(file)).ok().and_then(|d|serde_json::from_slice::<Value>(&d).ok());
 if let Some(key)=old.as_ref().and_then(|v|v.get("key")).and_then(Value::as_str).filter(|k|!k.is_empty()&&k.len()<32768){let mut manifest:Value=serde_json::from_slice(bytes).unwrap();manifest["key"]=json!(key);serde_json::to_vec_pretty(&manifest).unwrap()}else{bytes.to_vec()}
}
fn recover(root:&Path)->Result<(),String>{
 let work=root.join(".dd-update");if !work.exists(){return Ok(());}safe_path(&work)?;
 if work.join("committed").exists(){fs::remove_dir_all(work).map_err(|_|"Could not clean previous update")?;return Ok(());}
 let journal:Vec<String>=serde_json::from_slice(&fs::read(work.join("originals.json")).map_err(|_|"Interrupted update needs recovery")?).map_err(|_|"Invalid update journal")?;
 if journal.iter().any(|p|!FILES.iter().any(|(f,_)|p==f)){return Err("Invalid update journal".into());}
 for (file,_) in FILES{let dest=root.join(file);safe_path(&dest)?;let backup=work.join("backup").join(file);safe_path(&backup)?;
  if backup.exists(){if dest.exists(){fs::remove_file(&dest).map_err(|_|"Could not roll back companion")?;}fs::rename(backup,dest).map_err(|_|"Could not restore companion")?;}
  else if !journal.iter().any(|p|p==file)&&dest.exists(){fs::remove_file(dest).map_err(|_|"Could not roll back new companion file")?;}
 }
 fs::remove_dir_all(work).map_err(|_|"Could not clean update recovery")?;Ok(())
}
fn install(root:&Path,new:bool,fail_after:Option<usize>)->Result<(),String>{
 if !root.is_absolute()||root.components().any(|c|matches!(c,std::path::Component::ParentDir)){return Err("Choose an absolute extension folder".into());}
 safe_path(root)?;fs::create_dir_all(root).map_err(|_|"Extension folder is not writable")?;recover(root)?;
 if !new&&!recognized(root){return Err("This folder is not an existing Digital Discipline companion".into());}
 for (file,_) in FILES{safe_path(&root.join(file))?;}
 if FILES.iter().all(|(file,bytes)|fs::read(root.join(file)).is_ok_and(|actual|actual==contents(root,file,bytes))){return Ok(());}
 let work=root.join(".dd-update");fs::create_dir(&work).map_err(|_|"Could not stage companion")?;
 let originals:Vec<_>=FILES.iter().filter(|(f,_)|root.join(f).is_file()).map(|(f,_)|f.to_string()).collect();
 let prepare=(||->std::io::Result<()>{for (file,bytes) in FILES{let staged=work.join("new").join(file);fs::create_dir_all(staged.parent().unwrap())?;fs::write(staged,contents(root,file,bytes))?;}fs::write(work.join("originals.json"),serde_json::to_vec(&originals).unwrap())?;Ok(())})();
 if prepare.is_err(){let _=fs::remove_dir_all(&work);return Err("Could not stage companion files".into());}
 let commit=(||->std::io::Result<()>{for (i,(file,_)) in FILES.iter().enumerate(){if fail_after==Some(i){return Err(std::io::Error::other("injected update failure"));}
  let dest=root.join(file);fs::create_dir_all(dest.parent().unwrap())?;
  if dest.exists(){let backup=work.join("backup").join(file);fs::create_dir_all(backup.parent().unwrap())?;fs::rename(&dest,backup)?;}
  fs::rename(work.join("new").join(file),dest)?;
 }fs::write(work.join("committed"),b"complete")?;Ok(())})();
 if commit.is_err(){recover(root)?;return Err("Update failed; previous companion restored".into());}
 let _=fs::remove_dir_all(work);Ok(())
}
fn roots(db:&rusqlite::Connection)->Vec<PathBuf>{
 let mut result=vec![crate::data_dir().unwrap_or_default().join("browser-extension")];
 // Earlier installers loaded directly from the desktop resources. Preserve that exact location.
 if let Ok(exe)=std::env::current_exe(){if exe.parent().and_then(Path::file_name).is_some_and(|n|n=="bin"){if let Some(parent)=exe.parent().and_then(Path::parent){let old=parent.join("browser-extension");if recognized(&old){result.push(old);}}}}
 let custom:Vec<String>=serde_json::from_str(&features::read_setting(db,"companion_update_folders")).unwrap_or_default();
 result.extend(custom.into_iter().map(PathBuf::from));result.sort();result.dedup();result
}
pub async fn apply(s:Arc<AppState>)->Result<Value,String>{
 let folders={let db=s.db.lock().map_err(|_|"Storage unavailable")?;roots(&db)};
 tokio::task::spawn_blocking(move||{let _lock=UPDATE_LOCK.lock().map_err(|_|"Updater unavailable")?;for root in &folders{let fresh=!root.join("manifest.json").exists();install(root,fresh,None)?;}Ok(json!({"ok":true,"version":version(),"reload_ready":true,"preserves_profile":true}))}).await.map_err(|_|"Updater interrupted".to_string())?
}
pub async fn status(State(s):State<Arc<AppState>>)->ApiResult{let db=s.db.lock().unwrap();Ok(Json(json!({"ok":true,"version":version(),"folder":crate::data_dir().unwrap_or_default().join("browser-extension").to_string_lossy(),"managed_folders":roots(&db).iter().map(|p|p.to_string_lossy().to_string()).collect::<Vec<_>>(),"reload_preserves_identity":true})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct InstallInput{existing_folder:Option<String>}
pub async fn prepare(State(s):State<Arc<AppState>>,Json(p):Json<InstallInput>)->ApiResult{
 if let Some(folder)=p.existing_folder{let path=PathBuf::from(&folder);if folder.len()>1000||!path.is_absolute()||safe_path(&path).is_err()||!recognized(&path){return Err(err(StatusCode::BAD_REQUEST,"existing_companion_folder_required"));}
  let db=s.db.lock().unwrap();let mut folders:Vec<String>=serde_json::from_str(&features::read_setting(&db,"companion_update_folders")).unwrap_or_default();if !folders.contains(&folder){if folders.len()>=8{return Err(err(StatusCode::BAD_REQUEST,"too_many_update_folders"));}folders.push(folder);}
  db.execute("INSERT OR REPLACE INTO settings VALUES('companion_update_folders',?1)",params![serde_json::to_string(&folders).unwrap()]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 }
 let mut result=apply(Arc::clone(&s)).await.map_err(|_|err(StatusCode::CONFLICT,"companion_update_failed_previous_files_preserved"))?;result["folder"]=json!(crate::data_dir().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"folder_unavailable"))?.join("browser-extension").to_string_lossy());Ok(Json(result))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn update_preserves_owner_files_and_rolls_back_failure(){let root=std::env::temp_dir().join(format!("dd-companion-{}",uuid::Uuid::new_v4()));install(&root,true,None).unwrap();fs::write(root.join("owner-note.txt"),b"keep").unwrap();fs::write(root.join("popup.js"),b"old worker fixture").unwrap();assert!(install(&root,false,Some(6)).is_err());assert_eq!(fs::read(root.join("popup.js")).unwrap(),b"old worker fixture");install(&root,false,None).unwrap();assert_eq!(fs::read(root.join("owner-note.txt")).unwrap(),b"keep");assert!(root.join("src/playback-model.js").exists());fs::remove_dir_all(root).unwrap();}
 #[test]fn unrelated_folder_is_not_an_update_target(){let root=std::env::temp_dir().join(format!("dd-foreign-{}",uuid::Uuid::new_v4()));fs::create_dir(&root).unwrap();fs::write(root.join("manifest.json"),b"{}").unwrap();assert!(install(&root,false,None).is_err());assert_eq!(fs::read(root.join("manifest.json")).unwrap(),b"{}");fs::remove_dir_all(root).unwrap();}
 #[test]fn an_existing_manifest_public_key_is_preserved_for_extension_identity(){let root=std::env::temp_dir().join(format!("dd-key-{}",uuid::Uuid::new_v4()));install(&root,true,None).unwrap();let mut manifest:Value=serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();manifest["key"]=json!("QUJDRA==");manifest["version"]=json!("0.1.0");fs::write(root.join("manifest.json"),serde_json::to_vec(&manifest).unwrap()).unwrap();install(&root,false,None).unwrap();let updated:Value=serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();assert_eq!(updated["key"],"QUJDRA==");assert_eq!(updated["version"],version());fs::remove_dir_all(root).unwrap();}
}
