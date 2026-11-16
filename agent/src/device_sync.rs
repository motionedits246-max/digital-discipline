//! Optional file transport for two owner-enrolled desktops. Packets are encrypted and signed.
use crate::{backup, credentials, data_lifecycle, devices, err, features, now_ms, ApiResult, AppState};
use axum::{extract::State,http::StatusCode,Json};
use ring::{rand::SystemRandom,signature::{self,Ed25519KeyPair,KeyPair}};
use rusqlite::{params,Connection,OptionalExtension};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{collections::BTreeSet,fs,io::{Read,Write},path::{Path,PathBuf},
    sync::{atomic::{AtomicBool,Ordering},Arc}};
use uuid::Uuid;
use zeroize::Zeroizing;

const MAGIC:&[u8]=b"DDSYNC\x01";
const WINDOW:i64=30*86_400_000;
const MAX_RECORDS:usize=50_000;
const MAX_BYTES:u64=24*1024*1024;
const SIGNATURE_BYTES:usize=64;
static BUSY:AtomicBool=AtomicBool::new(false);
struct Lease;
impl Lease {
    fn take()->Result<Self,&'static str>{
        BUSY.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst)
            .map(|_|Self).map_err(|_|"sync_already_running")
    }
}
impl Drop for Lease{fn drop(&mut self){BUSY.store(false,Ordering::SeqCst);}}

pub fn migrate(db:&Connection)->rusqlite::Result<()>{
    db.execute_batch("CREATE TABLE IF NOT EXISTS sync_peers(
        id TEXT PRIMARY KEY, name TEXT NOT NULL, key_hex TEXT NOT NULL,
        revision INTEGER NOT NULL DEFAULT 0, revoked INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS sync_settings(
        id INTEGER PRIMARY KEY CHECK(id=1), folder TEXT NOT NULL, enabled INTEGER NOT NULL,
        key_revision TEXT NOT NULL, last_sync_ms INTEGER NOT NULL DEFAULT 0,
        last_error TEXT NOT NULL DEFAULT '', revision INTEGER NOT NULL DEFAULT 0);")
}

fn data_path(name:&str)->Result<PathBuf,&'static str>{
    Ok(crate::data_dir().map_err(|_|"sync_local_storage_unavailable")?.join(name))
}
fn pair_key(create:bool)->Result<Ed25519KeyPair,&'static str>{
    let path=data_path("device-signing.key")?;
    let bytes=match fs::read(&path){
        Ok(sealed)=>Zeroizing::new(credentials::unprotect_secret(&sealed).map_err(|_|"device_key_unavailable")?),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound && create=>{
            let secret=Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).map_err(|_|"device_key_generation_failed")?;
            let bytes=Zeroizing::new(secret.as_ref().to_vec());
            let protected=credentials::protect_secret(&bytes).map_err(|_|"sync_requires_windows_key_protection")?;
            let mut file=fs::OpenOptions::new().write(true).create_new(true).open(path)
                .map_err(|_|"device_key_write_failed")?;
            file.write_all(&protected).and_then(|_|file.sync_all()).map_err(|_|"device_key_write_failed")?;
            bytes
        }
        _=>return Err("device_key_unavailable"),
    };
    Ed25519KeyPair::from_pkcs8(&bytes).map_err(|_|"device_key_invalid")
}
fn failure(code:&str)->(StatusCode,String){err(StatusCode::BAD_REQUEST,code)}
async fn work(job:impl FnOnce()->Result<Value,&'static str>+Send+'static)->ApiResult{
    tokio::task::spawn_blocking(job).await.map_err(|_|failure("sync_worker_failed"))?
        .map(Json).map_err(failure)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {enabled:bool,folder:Option<String>,passphrase:Option<String>}
pub async fn configure(State(state):State<Arc<AppState>>,Json(input):Json<Setup>)->ApiResult{
    work(move||{
        let _lease=Lease::take()?;
        if !input.enabled {
            let db=state.db.lock().unwrap();
            let key=db.query_row("SELECT key_revision FROM sync_settings WHERE id=1",[],|r|r.get::<_,String>(0))
                .optional().map_err(|_|"sync_storage_failed")?;
            db.execute("UPDATE sync_settings SET enabled=0,key_revision='' WHERE id=1",[])
                .map_err(|_|"sync_storage_failed")?;
            if let Some(key)=key.filter(|key|Uuid::parse_str(key).is_ok()) {
                let _=fs::remove_file(data_path(&format!("sync-secret-{key}.key"))?);
            }
            return Ok(json!({"ok":true}));
        }
        let path=PathBuf::from(input.folder.ok_or("choose_a_shared_folder")?);
        if !path.is_absolute(){return Err("choose_an_absolute_shared_folder");}
        let path=fs::canonicalize(path).map_err(|_|"shared_folder_unavailable")?;
        if !path.is_dir(){return Err("shared_folder_unavailable");}
        let secret=Zeroizing::new(input.passphrase.ok_or("sync_passphrase_required")?);
        if secret.chars().count()<12||secret.len()>1024{return Err("sync_passphrase_needs_12_characters");}
        let protected=credentials::protect_secret(secret.as_bytes()).map_err(|_|"sync_requires_windows_key_protection")?;
        pair_key(true)?;
        let revision=Uuid::new_v4().to_string();
        let keyfile=data_path(&format!("sync-secret-{revision}.key"))?;
        fs::write(&keyfile,protected).map_err(|_|"sync_secret_write_failed")?;
        let db=state.db.lock().unwrap();
        let old=db.query_row("SELECT key_revision FROM sync_settings WHERE id=1",[],|r|r.get::<_,String>(0))
            .optional().map_err(|_|"sync_storage_failed")?;
        if db.execute("INSERT INTO sync_settings(id,folder,enabled,key_revision) VALUES(1,?1,1,?2)
            ON CONFLICT(id) DO UPDATE SET folder=excluded.folder,enabled=1,key_revision=excluded.key_revision,last_error=''",
            params![path.to_string_lossy(),revision]).is_err(){
            let _=fs::remove_file(keyfile);
            return Err("sync_storage_failed");
        }
        if let Some(key)=old.filter(|key|Uuid::parse_str(key).is_ok()) {
            let _=fs::remove_file(data_path(&format!("sync-secret-{key}.key"))?);
        }
        Ok(json!({"ok":true}))
    }).await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {code:String,name:String,revoke:bool}
pub async fn peer(State(state):State<Arc<AppState>>,Json(input):Json<Peer>)->ApiResult{
    let _lease=Lease::take().map_err(failure)?;
    let (id,key)=if input.revoke{(input.code.as_str(),"")}else{
        input.code.trim().split_once(':').ok_or_else(||failure("invalid_device_code"))?
    };
    if Uuid::parse_str(id).is_err() || !input.revoke &&
        (key.len()!=64 || !key.bytes().all(|byte|byte.is_ascii_hexdigit()) ||
         input.name.trim().is_empty() || input.name.len()>80 || input.name.chars().any(char::is_control)) {
        return Err(failure("invalid_device_code"));
    }
    let db=state.db.lock().unwrap();
    if features::read_setting(&db,"device_id")==id{return Err(failure("cannot_link_this_device_to_itself"));}
    let tx=db.unchecked_transaction().map_err(|_|failure("sync_storage_failed"))?;
    if input.revoke{
        tx.execute("UPDATE sync_peers SET revoked=1 WHERE id=?1",[id]).map_err(|_|failure("sync_storage_failed"))?;
        tx.execute("DELETE FROM device_sessions WHERE device_id=?1",[id]).map_err(|_|failure("sync_storage_failed"))?;
        tx.execute("DELETE FROM linked_devices WHERE device_id=?1",[id]).map_err(|_|failure("sync_storage_failed"))?;
    }else{
        tx.execute("INSERT INTO sync_peers(id,name,key_hex) VALUES(?1,?2,?3)
            ON CONFLICT(id) DO UPDATE SET name=excluded.name,
            revision=CASE WHEN key_hex=excluded.key_hex THEN revision ELSE 0 END,key_hex=excluded.key_hex,revoked=0",
            params![id,input.name.trim(),key.to_ascii_lowercase()]).map_err(|_|failure("sync_storage_failed"))?;
    }
    tx.commit().map_err(|_|failure("sync_storage_failed"))?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet{id:String,revision:i64,created_ms:i64,records:Vec<devices::Record>}
fn file_path(folder:&Path,id:&str)->PathBuf{folder.join(format!("dd-sync-{id}.dds"))}
fn publish(folder:&Path,id:&str,wire:&[u8],key:&Ed25519KeyPair)->Result<(),&'static str>{
    let path=file_path(folder,id);
    let temp=folder.join(format!("dd-sync-{}.partial",Uuid::new_v4()));
    let result=(||->std::io::Result<()>{
        let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.write_all(key.sign(wire).as_ref())?;
        file.write_all(wire)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp,path)
    })();
    if result.is_err(){let _=fs::remove_file(temp);return Err("shared_folder_write_failed");}
    Ok(())
}
fn read_packet(path:PathBuf,key:&str,secret:&str)->Result<Option<Packet>,&'static str>{
    let metadata=match fs::symlink_metadata(&path){
        Ok(metadata)=>metadata,
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),
        Err(_)=>return Err("sync_packet_unavailable"),
    };
    if !metadata.file_type().is_file() || metadata.len()>MAX_BYTES{return Err("invalid_sync_packet");}
    let mut bytes=Vec::new();
    fs::File::open(path).map_err(|_|"sync_packet_unavailable")?.take(MAX_BYTES+1)
        .read_to_end(&mut bytes).map_err(|_|"sync_packet_unavailable")?;
    if bytes.len()<=SIGNATURE_BYTES || bytes.len() as u64>MAX_BYTES{return Err("invalid_sync_packet");}
    let (sig,wire)=bytes.split_at(SIGNATURE_BYTES);
    signature::UnparsedPublicKey::new(&signature::ED25519,hex::decode(key).map_err(|_|"invalid_device_key")?)
        .verify(wire,sig).map_err(|_|"sync_signature_rejected")?;
    serde_json::from_slice(&backup::open_payload(MAGIC,wire,secret)?)
        .map(Some).map_err(|_|"invalid_sync_packet")
}
fn accept(db:&Connection,id:&str,name:&str,packet:Packet,now:i64)->Result<(),&'static str>{
    if packet.id!=id || packet.revision<=0 || packet.created_ms<now-WINDOW || packet.created_ms>now+300_000
        || packet.records.len()>MAX_RECORDS || packet.records.iter().any(|row|!devices::record_valid(row,now))
        || packet.records.iter().map(|row|&row.session_id).collect::<BTreeSet<_>>().len()!=packet.records.len(){
        return Err("invalid_sync_records");
    }
    let tx=db.unchecked_transaction().map_err(|_|"sync_storage_failed")?;
    let changed=tx.execute("UPDATE sync_peers SET revision=?2 WHERE id=?1 AND revoked=0 AND revision<?2",
        params![id,packet.revision]).map_err(|_|"sync_storage_failed")?;
    if changed==0 {return Ok(());}
    tx.execute("DELETE FROM device_sessions WHERE device_id=?1",[id]).map_err(|_|"sync_storage_failed")?;
    for row in packet.records {
        if features::excluded(&tx,&row.app,row.domain.as_deref()) ||
            data_lifecycle::rejection(&tx,row.start_ms,Some(&format!("app:{}",row.app.to_ascii_lowercase())))
                .map_err(|_|"sync_privacy_check_failed")?.is_some() ||
            row.domain.as_ref().is_some_and(|host|data_lifecycle::rejection(&tx,row.start_ms,Some(&format!("domain:{host}")))
                .map_or(true,|result|result.is_some())){
            continue;
        }
        tx.execute("INSERT INTO device_sessions VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![id,row.session_id,row.start_ms,row.end_ms,row.app,row.idle,row.domain])
            .map_err(|_|"sync_storage_failed")?;
    }
    tx.execute("INSERT INTO linked_devices VALUES(?1,?2,?3) ON CONFLICT(device_id) DO UPDATE SET
        display_name=excluded.display_name,last_import_ms=excluded.last_import_ms",
        params![id,name,now]).map_err(|_|"sync_storage_failed")?;
    tx.commit().map_err(|_|"sync_storage_failed")
}

fn sync(state:&AppState)->Result<(),&'static str>{
    let _lease=Lease::take()?;
    let (path,folder,secret_id,id,revision)={
        let db=state.db.lock().unwrap();
        let options=db.query_row("SELECT folder,key_revision,revision FROM sync_settings WHERE id=1 AND enabled=1",[],
            |r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))
            .optional().map_err(|_|"sync_storage_failed")?;
        let Some((folder,secret,revision))=options else{return Ok(());};
        if Uuid::parse_str(&secret).is_err(){return Err("sync_secret_unavailable");}
        let revision=revision.checked_add(1).ok_or("sync_revision_exhausted")?;
        db.execute("UPDATE sync_settings SET revision=?1 WHERE id=1",[revision]).map_err(|_|"sync_storage_failed")?;
        (db.path().ok_or("sync_storage_failed")?.to_owned(),PathBuf::from(folder),secret,
         features::read_setting(&db,"device_id"),revision)
    };
    if Uuid::parse_str(&id).is_err(){return Err("invalid_device_identity");}
    let sealed=fs::read(data_path(&format!("sync-secret-{secret_id}.key"))?).map_err(|_|"sync_secret_unavailable")?;
    let secret=Zeroizing::new(credentials::unprotect_secret(&sealed).map_err(|_|"sync_secret_unavailable")?);
    let secret=std::str::from_utf8(&secret).map_err(|_|"sync_secret_unavailable")?;
    let now=now_ms();
    let db=Connection::open_with_flags(path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_|"sync_snapshot_unavailable")?;
    let mut q=db.prepare("SELECT id,start_ms,end_ms,app,idle,domain FROM sessions
        WHERE end_ms>?1 AND end_ms>start_ms ORDER BY start_ms,id LIMIT ?2").map_err(|_|"sync_snapshot_unavailable")?;
    let records=q.query_map(params![now-WINDOW,MAX_RECORDS+1],|r|Ok(devices::Record{
        session_id:r.get(0)?,start_ms:r.get::<_,i64>(1)?.max(now-WINDOW),end_ms:r.get(2)?,
        app:r.get(3)?,idle:r.get(4)?,domain:r.get(5)?})).map_err(|_|"sync_snapshot_unavailable")?
        .collect::<rusqlite::Result<Vec<_>>>().map_err(|_|"sync_snapshot_unavailable")?;
    if records.len()>MAX_RECORDS{return Err("sync_history_exceeds_50000_records");}
    let payload=serde_json::to_vec(&Packet{id:id.clone(),revision,created_ms:now,records})
        .map_err(|_|"sync_encoding_failed")?;
    let wire=backup::seal_payload(MAGIC,payload,secret)?;
    if wire.len() as u64+SIGNATURE_BYTES as u64>MAX_BYTES{return Err("sync_packet_too_large");}
    publish(&folder,&id,&wire,&pair_key(false)?)?;
    let mut peers=db.prepare("SELECT id,name,key_hex FROM sync_peers WHERE revoked=0 LIMIT 30")
        .map_err(|_|"sync_storage_failed")?;
    let peers=peers.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?)))
        .map_err(|_|"sync_storage_failed")?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|"sync_storage_failed")?;
    let mut warning="";
    for (peer,name,key) in peers {
        match read_packet(file_path(&folder,&peer),&key,secret) {
            Ok(Some(packet))=>accept(&state.db.lock().unwrap(),&peer,&name,packet,now)?,
            Ok(None)=>{},
            Err(error)=>warning=error,
        }
    }
    let db=state.db.lock().unwrap();
    db.execute("UPDATE sync_settings SET last_sync_ms=?1,last_error=?2 WHERE id=1",params![now,warning])
        .map_err(|_|"sync_storage_failed")?;
    Ok(())
}
pub async fn now(State(state):State<Arc<AppState>>)->ApiResult{
    work(move||{sync(&state)?;Ok(json!({"ok":true}))}).await
}
pub async fn scheduler(state:Arc<AppState>){
    let mut timer=tokio::time::interval(std::time::Duration::from_secs(180));
    loop{
        timer.tick().await;
        if state.stopping.load(Ordering::SeqCst){return;}
        let state=state.clone();
        let _=tokio::task::spawn_blocking(move||{
            if let Err(code)=sync(&state){
                if code!="sync_already_running"{
                    let _=state.db.lock().unwrap().execute("UPDATE sync_settings SET last_error=?1 WHERE id=1",[code]);
                }
            }
        }).await;
    }
}
pub async fn status(State(state):State<Arc<AppState>>)->ApiResult{
    work(move||{
        let db=state.db.lock().unwrap();
        let id=features::read_setting(&db,"device_id");
        let code=pair_key(false).ok().map(|key|format!("{}:{}",id,hex::encode(key.public_key().as_ref())));
        let state=db.query_row("SELECT folder,enabled,last_sync_ms,last_error FROM sync_settings WHERE id=1",[],
            |r|Ok(json!({"folder":r.get::<_,String>(0)?,"enabled":r.get::<_,bool>(1)?,"last_sync_ms":r.get::<_,i64>(2)?,
                "last_error":r.get::<_,String>(3)?}))).optional().map_err(|_|"sync_storage_failed")?;
        let mut q=db.prepare("SELECT id,name,revision,revoked FROM sync_peers ORDER BY name LIMIT 30")
            .map_err(|_|"sync_storage_failed")?;
        let peers=q.query_map([],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,
            "revision":r.get::<_,i64>(2)?,"revoked":r.get::<_,bool>(3)?})))
            .map_err(|_|"sync_storage_failed")?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|"sync_storage_failed")?;
        Ok(json!({"ok":true,"state":state,"code":code,"peers":peers}))
    }).await
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn newer_snapshots_remove_deleted_rows_and_revocation_rejects_replays(){
        let db=Connection::open_in_memory().unwrap();
        crate::initialize_schema(&db).unwrap();
        let id=Uuid::new_v4().to_string();
        let now=now_ms();
        db.execute("INSERT INTO sync_peers(id,name,key_hex) VALUES(?1,'Laptop','')",[&id]).unwrap();
        let make=|revision,has_row|Packet{id:id.clone(),revision,created_ms:now,records:if has_row{
            vec![devices::Record{session_id:"e7051421-b659-4a47-987f-fec6e436c4e0".into(),start_ms:now-1000,
                end_ms:now,app:"editor.exe".into(),idle:false,domain:None}]
        }else{vec![]}};
        accept(&db,&id,"Laptop",make(1,true),now).unwrap();
        accept(&db,&id,"Laptop",make(2,false),now).unwrap();
        accept(&db,&id,"Laptop",make(1,true),now).unwrap();
        db.execute("UPDATE sync_peers SET revoked=1",[]).unwrap();
        accept(&db,&id,"Laptop",make(3,true),now).unwrap();
        assert_eq!(db.query_row("SELECT count(*) FROM device_sessions",[],|r|r.get::<_,u32>(0)).unwrap(),0);
    }
}
