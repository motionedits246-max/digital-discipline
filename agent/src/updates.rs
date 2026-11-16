//! Explicit update checks against the product's HTTPS release manifest.
use crate::{err,ApiResult};
use axum::{http::StatusCode,Json};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{path::Path,sync::atomic::{AtomicBool,Ordering},time::Duration};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

const SITE:&str="https://digital-discipline-windows.codecut-official.chatgpt.site";
const MAX_INSTALLER:u64=32*1024*1024;
static DOWNLOADING:AtomicBool=AtomicBool::new(false);
struct DownloadLease;
impl Drop for DownloadLease{
    fn drop(&mut self){DOWNLOADING.store(false,Ordering::SeqCst);}
}

#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Release{
    schema:u32,
    version:String,
    companion_version:String,
    protocol:String,
    filename:String,
    bytes:u64,
    sha256:String,
    signing:String,
    signer_thumbprint:String,
    notes:String,
}
fn version(text:&str)->Option<[u16;3]>{
    let mut fields=text.split('.');
    let mut number=[0;3];
    for field in &mut number{
        let token=fields.next()?;
        if token.is_empty()||token.len()>4||!token.bytes().all(|byte|byte.is_ascii_digit()){return None;}
        *field=token.parse().ok()?;
    }
    fields.next().is_none().then_some(number)
}
fn client()->Result<reqwest::Client,&'static str>{
    reqwest::Client::builder().https_only(true).redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(60)).build().map_err(|_|"update_client_unavailable")
}
async fn metadata(client:&reqwest::Client)->Result<Release,&'static str>{
    let mut response=client.get(format!("{SITE}/downloads/latest.json")).send().await
        .map_err(|_|"release_server_unavailable")?.error_for_status().map_err(|_|"release_manifest_unavailable")?;
    let mut bytes=Vec::new();
    while let Some(chunk)=response.chunk().await.map_err(|_|"release_manifest_unavailable")? {
        if bytes.len()+chunk.len()>32768{return Err("release_manifest_too_large");}
        bytes.extend_from_slice(&chunk);
    }
    let release:Release=serde_json::from_slice(&bytes).map_err(|_|"invalid_release_manifest")?;
    if release.schema!=1 || version(&release.version).is_none() || version(&release.companion_version).is_none()
        || release.protocol!=crate::domain::SCHEMA_VERSION
        || release.filename!=format!("Digital-Discipline-{}-x64-setup.exe",release.version)
        || !(100_000..=MAX_INSTALLER).contains(&release.bytes)
        || release.sha256.len()!=64 || !release.sha256.bytes().all(|byte|byte.is_ascii_hexdigit())
        || release.signer_thumbprint.len()!=40
        || !release.signer_thumbprint.bytes().all(|byte|byte.is_ascii_hexdigit())
        || !matches!(release.signing.as_str(),"development"|"trusted")
        || release.notes.len()>4000{
        return Err("incompatible_release_manifest");
    }
    Ok(release)
}
fn failure(code:&str)->(StatusCode,String){err(StatusCode::BAD_REQUEST,code)}
pub async fn check()->ApiResult{
    let release=metadata(&client().map_err(failure)?).await.map_err(failure)?;
    let installed=env!("CARGO_PKG_VERSION");
    let available=version(&release.version)>version(installed);
    Ok(Json(json!({"ok":true,"installed":installed,"available":available,"release":release,
        "expected_companion":crate::companion::version(),"protocol":crate::domain::SCHEMA_VERSION})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verify {version:String,sha256:String}

#[cfg(windows)]
async fn authenticode(path:&Path)->Result<Value,&'static str>{
    let system_root=std::env::var_os("SystemRoot").ok_or("windows_signature_check_failed")?;
    let program=std::path::PathBuf::from(system_root)
        .join("System32").join("WindowsPowerShell").join("v1.0").join("powershell.exe");
    let mut command=tokio::process::Command::new(program);
    command.creation_flags(0x08000000).kill_on_drop(true)
        .args(["-NoProfile","-NonInteractive","-Command",
            "$ErrorActionPreference='Stop'; $s=Get-AuthenticodeSignature -LiteralPath $env:DD_VERIFY_FILE; @{status=$s.Status.ToString();signed=[bool]$s.SignerCertificate;publisher=$s.SignerCertificate.Subject;thumbprint=$s.SignerCertificate.Thumbprint} | ConvertTo-Json -Compress"])
        .env("DD_VERIFY_FILE",path);
    let result=tokio::time::timeout(Duration::from_secs(30),command.output()).await
        .map_err(|_|"windows_signature_check_timed_out")?.map_err(|_|"windows_signature_check_failed")?;
    if !result.status.success(){return Err("windows_signature_check_failed");}
    serde_json::from_slice(&result.stdout).map_err(|_|"windows_signature_check_failed")
}
#[cfg(not(windows))]
async fn authenticode(_path:&Path)->Result<Value,&'static str>{Err("verify_installer_on_windows")}

pub async fn verify(Json(input):Json<Verify>)->ApiResult{
    if !cfg!(windows){return Err(failure("verify_installer_on_windows"));}
    DOWNLOADING.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst)
        .map_err(|_|failure("installer_verification_running"))?;
    let _lease=DownloadLease;
    let client=client().map_err(failure)?;
    let release=metadata(&client).await.map_err(failure)?;
    if release.version!=input.version || !release.sha256.eq_ignore_ascii_case(&input.sha256){
        return Err(failure("release_changed_check_again"));
    }
    let root=crate::data_dir().map_err(|_|failure("update_folder_unavailable"))?;
    let folder=root.join("updates");
    let staging=root.join("update-staging");
    for directory in [&folder,&staging] {
        tokio::fs::create_dir_all(directory).await.map_err(|_|failure("update_folder_unavailable"))?;
    }
    let path=staging.join(format!("{}.exe",Uuid::new_v4()));
    let result=async {
        let mut response=client.get(format!("{SITE}/downloads/{}",release.filename)).send().await
            .map_err(|_|"installer_download_failed")?.error_for_status().map_err(|_|"installer_download_failed")?;
        let mut file=tokio::fs::OpenOptions::new().write(true).create_new(true).open(&path).await
            .map_err(|_|"installer_save_failed")?;
        let mut hash=Sha256::new();
        let mut received=0;
        while let Some(chunk)=response.chunk().await.map_err(|_|"installer_download_failed")? {
            received+=chunk.len() as u64;
            if received>release.bytes{return Err("installer_size_mismatch");}
            hash.update(&chunk);
            file.write_all(&chunk).await.map_err(|_|"installer_save_failed")?;
        }
        file.sync_all().await.map_err(|_|"installer_save_failed")?;
        drop(file);
        if received!=release.bytes || !hex::encode(hash.finalize()).eq_ignore_ascii_case(&release.sha256){
            return Err("installer_checksum_mismatch");
        }
        let signature=authenticode(&path).await?;
        let status=signature["status"].as_str().unwrap_or("");
        let thumbprint=signature["thumbprint"].as_str().unwrap_or("");
        let trusted=release.signing=="trusted";
        let status_allowed=match release.signing.as_str(){
            "trusted"=>status=="Valid",
            "development"=>matches!(status,"Valid"|"NotTrusted"|"UnknownError"),
            _=>false,
        };
        if signature["signed"]!=true || !thumbprint.eq_ignore_ascii_case(&release.signer_thumbprint) || !status_allowed {
            return Err("installer_signature_rejected");
        }
        let filename=format!("{}-{}",Uuid::new_v4(),release.filename);
        tokio::fs::rename(&path,folder.join(&filename)).await.map_err(|_|"installer_save_failed")?;
        Ok(json!({"ok":true,"filename":filename,"signature":signature,
            "checksum_verified":true,"trusted_publisher":trusted}))
    }.await;
    if result.is_err(){let _=tokio::fs::remove_file(&path).await;}
    result.map(Json).map_err(failure)
}

#[cfg(test)]
mod tests{
    #[test]
    fn version_order_is_numeric_and_rejects_path_components(){
        use super::version;
        assert!(version("0.10.0")>version("0.9.9"));
        for value in ["../0.5.0","0.5","1.2.3.4","v1.2.3","1.-2.0"]{assert!(version(value).is_none());}
    }
}
