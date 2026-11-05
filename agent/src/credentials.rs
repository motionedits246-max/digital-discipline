//! Current-user scoped Windows DPAPI credential vault for AI Coach.
//! Secrets never enter the activity SQLite database, browsing extension or exports.
//! Encrypted blobs are only recoverable by the same Windows user profile (subject
//! to the normal Windows DPAPI roaming/recovery behavior). No plaintext disk writes.
use std::{fs,path::PathBuf};
use axum::{extract::{Path,State},http::StatusCode,Json};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::{AppState,ApiResult,err};

pub fn supported_provider(p:&str)->bool {
    matches!(p,"openai"|"anthropic"|"gemini"|"groq"|"openrouter")
}
fn location(provider:&str)->Result<PathBuf,String>{
    if !supported_provider(provider){return Err("unsupported_provider".to_string());}
    let root=std::env::var_os("LOCALAPPDATA").ok_or("local_app_data_unavailable")?;
    Ok(PathBuf::from(root).join("DigitalDiscipline").join("coach-credentials").join(format!("{provider}.dpapi")))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveCredential {provider:String,api_key:String}

pub async fn list(State(_state):State<Arc<AppState>>)->ApiResult {
    let providers=["openai","anthropic","gemini","groq","openrouter"]
        .into_iter().filter(|p|location(p).is_ok_and(|path|path.is_file())).collect::<Vec<_>>();
    Ok(Json(json!({"ok":true,"providers":providers,"storage":"Windows DPAPI current-user"})))
}
pub async fn save(State(_state):State<Arc<AppState>>,Json(mut p):Json<SaveCredential>)->ApiResult{
    use zeroize::Zeroize;
    let result=(||{
        if p.api_key.len()<8||p.api_key.len()>512||p.api_key.bytes().any(|c|c==b'\n'||c==b'\r'||c==0){
            return Err(err(StatusCode::BAD_REQUEST,"invalid_provider_key"));
        }
        let path=location(&p.provider).map_err(|_|err(StatusCode::BAD_REQUEST,"unsupported_provider"))?;
        let encrypted=protect_secret(p.api_key.as_bytes()).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"credential_encryption_failed"))?;
        fs::create_dir_all(path.parent().ok_or_else(||err(StatusCode::INTERNAL_SERVER_ERROR,"invalid_vault_path"))?)
            .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"credential_storage_failed"))?;
        fs::write(&path,encrypted).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"credential_storage_failed"))?;
        Ok(Json(json!({"ok":true,"provider":p.provider,"saved":true})))
    })();
    p.api_key.zeroize();
    result
}
pub async fn remove(Path(provider):Path<String>,State(_state):State<Arc<AppState>>)->ApiResult {
    let path=location(&provider).map_err(|_|err(StatusCode::BAD_REQUEST,"unsupported_provider"))?;
    let removed=if path.exists(){
        fs::remove_file(path).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"credential_delete_failed"))?;
        true
    }else{false};
    Ok(Json(json!({"ok":true,"provider":provider,"removed":removed})))
}
pub fn load(provider:&str)->Result<String,String>{
    let path=location(provider)?;
    let encrypted=fs::read(path).map_err(|_|"credential_missing".to_string())?;
    if encrypted.len()>8192{return Err("credential_corrupt".to_string());}
    let raw=unprotect_secret(&encrypted).map_err(|_|"credential_decryption_failed".to_string())?;
    let key=String::from_utf8(raw).map_err(|_|"credential_corrupt".to_string())?;
    if key.len()<8||key.len()>512{return Err("credential_corrupt".to_string());}
    Ok(key)
}
#[cfg(windows)]
fn protect_secret(bytes:&[u8])->Result<Vec<u8>,String>{
    use std::ptr::{null,null_mut};
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB,CryptProtectData,CRYPTPROTECT_UI_FORBIDDEN};
    let mut input=CRYPT_INTEGER_BLOB { cbData:bytes.len() as u32, pbData:bytes.as_ptr() as *mut u8 };
    let mut output=CRYPT_INTEGER_BLOB { cbData:0,pbData:null_mut() };
    // SAFETY: valid input slice, DPAPI allocates output with LocalAlloc, freed once.
    let succeeded=unsafe{CryptProtectData(&mut input,null(),null(),null(),null(),CRYPTPROTECT_UI_FORBIDDEN,&mut output)};
    if succeeded==0{return Err("dpapi_protect_failed".into());}
    if output.pbData.is_null()||output.cbData==0{return Err("dpapi_empty_output".into());}
    let encrypted=unsafe{std::slice::from_raw_parts(output.pbData,output.cbData as usize).to_vec()};
    unsafe{LocalFree(output.pbData as *mut std::ffi::c_void)};
    Ok(encrypted)
}
#[cfg(windows)]
fn unprotect_secret(bytes:&[u8])->Result<Vec<u8>,String>{
    use std::ptr::null_mut;
    use std::sync::atomic::{compiler_fence,Ordering};
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB,CryptUnprotectData,CRYPTPROTECT_UI_FORBIDDEN};
    let mut input=CRYPT_INTEGER_BLOB { cbData:bytes.len() as u32,pbData:bytes.as_ptr() as *mut u8 };
    let mut output=CRYPT_INTEGER_BLOB { cbData:0,pbData:null_mut() };
    // SAFETY: DPAPI returns LocalAlloc buffer. Wipe plaintext before freeing it.
    let succeeded=unsafe{CryptUnprotectData(&mut input,null_mut(),null_mut(),null_mut(),null_mut(),CRYPTPROTECT_UI_FORBIDDEN,&mut output)};
    if succeeded==0{return Err("dpapi_unprotect_failed".into());}
    if output.pbData.is_null()||output.cbData==0{return Err("dpapi_empty_output".into());}
    let secret=unsafe{std::slice::from_raw_parts(output.pbData,output.cbData as usize).to_vec()};
    unsafe{
        for i in 0..output.cbData as usize{std::ptr::write_volatile(output.pbData.add(i),0);}
        compiler_fence(Ordering::SeqCst);
        LocalFree(output.pbData as *mut std::ffi::c_void);
    }
    Ok(secret)
}
#[cfg(not(windows))]
fn protect_secret(_bytes:&[u8])->Result<Vec<u8>,String>{Err("windows_only".into())}
#[cfg(not(windows))]
fn unprotect_secret(_bytes:&[u8])->Result<Vec<u8>,String>{Err("windows_only".into())}

#[cfg(test)] mod tests{
    use super::*;
    #[test] fn provider_names_are_allowlisted(){
        assert!(supported_provider("gemini"));
        assert!(supported_provider("openrouter"));
        for bad in ["../openai","openai/evil","OLLAMA","","other","openai\\evil"] {assert!(!supported_provider(bad));}
    }
    #[test] #[cfg(windows)] fn user_scoped_dpapi_roundtrip(){
        let text=b"dummy_test_key_for_local_windows_dpapi_only";
        let ciphertext=protect_secret(text).unwrap();
        assert!(!ciphertext.windows(text.len()).any(|x|x==text));
        assert_eq!(unprotect_secret(&ciphertext).unwrap(),text);
        let mut corrupted=ciphertext;corrupted[corrupted.len()/2]^=0x0f;
        assert!(unprotect_secret(&corrupted).is_err());
    }
}
