//! Reusable, local focus settings with explicit review before a session begins.
use crate::{err, features, now_ms, ApiResult, AppState};
use axum::{extract::State,http::StatusCode,Json};
use rusqlite::{params,Connection,OptionalExtension};
use serde::{Deserialize,Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub fn migrate(db:&Connection)->rusqlite::Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS focus_presets(
        id TEXT PRIMARY KEY, name TEXT NOT NULL, configuration TEXT NOT NULL,
        revision INTEGER NOT NULL DEFAULT 1);")
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    id:Option<String>, revision:Option<i64>, name:String, profile_id:Option<String>,
    minutes:u32, difficulty:String, block_domains:Vec<String>, native_apps:Vec<String>,
    minimize_selected_apps:bool,
    #[serde(default)] remove:bool,
}
fn storage(_:rusqlite::Error)->(StatusCode,String){
    err(StatusCode::INTERNAL_SERVER_ERROR,"preset_storage_failed")
}
pub async fn list(State(state):State<Arc<AppState>>)->ApiResult{
    let db=state.db.lock().unwrap();
    let mut q=db.prepare("SELECT id,configuration,revision FROM focus_presets ORDER BY name LIMIT 100").map_err(storage)?;
    let presets=q.query_map([],|r|{
        let mut config=serde_json::from_str::<serde_json::Value>(&r.get::<_,String>(1)?)
            .map_err(|_|rusqlite::Error::InvalidQuery)?;
        config["id"]=json!(r.get::<_,String>(0)?);
        config["revision"]=json!(r.get::<_,i64>(2)?);
        Ok(config)
    }).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
    Ok(Json(json!({"ok":true,"presets":presets})))
}
pub async fn save(State(state):State<Arc<AppState>>,Json(input):Json<Preset>)->ApiResult{
    if input.id.as_ref().is_some_and(|id|Uuid::parse_str(id).is_err()){
        return Err(err(StatusCode::BAD_REQUEST,"invalid_preset"));
    }
    let db=state.db.lock().unwrap();
    if input.remove {
        if db.execute("DELETE FROM focus_presets WHERE id=?1 AND revision=?2",
            params![input.id,input.revision]).map_err(storage)?==0 {
            return Err(err(StatusCode::CONFLICT,"preset_changed"));
        }
        return Ok(Json(json!({"ok":true})));
    }
    if input.name.trim().is_empty()||input.name.chars().count()>80||input.name.chars().any(char::is_control)
        || !(1..=240).contains(&input.minutes)||!matches!(input.difficulty.as_str(),"Gentle"|"Balanced"|"Strict")
        || input.block_domains.len()>15||input.block_domains.iter().any(|name|!crate::domain::valid_domain(name))
        || input.native_apps.len()>20||input.native_apps.iter().any(|name|!crate::inbox::safe_native_app(name))
        || input.minimize_selected_apps && (input.difficulty!="Strict" || input.native_apps.is_empty())
    {return Err(err(StatusCode::BAD_REQUEST,"invalid_preset"));}
    if let Some(profile)=&input.profile_id {
        let active:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM workflow_profiles WHERE id=?1 AND archived=0)",
            [profile],|r|r.get(0)).map_err(storage)?;
        if !active{return Err(err(StatusCode::BAD_REQUEST,"profile_not_found"));}
    }
    let config=serde_json::to_string(&input).unwrap();
    let id=input.id.clone().unwrap_or_else(||Uuid::new_v4().to_string());
    if input.id.is_some(){
        if db.execute("UPDATE focus_presets SET name=?2,configuration=?3,revision=revision+1 WHERE id=?1 AND revision=?4",
            params![id,input.name.trim(),config,input.revision]).map_err(storage)?==0{
            return Err(err(StatusCode::CONFLICT,"preset_changed"));
        }
    }else{
        db.execute("INSERT INTO focus_presets(id,name,configuration) VALUES(?1,?2,?3)",
            params![id,input.name.trim(),config]).map_err(storage)?;
    }
    Ok(Json(json!({"ok":true,"id":id})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start{id:String,revision:i64}
pub async fn start(State(state):State<Arc<AppState>>,Json(input):Json<Start>)->ApiResult{
    let db=state.db.lock().unwrap();
    let tx=db.unchecked_transaction().map_err(storage)?;
    let encoded=tx.query_row("SELECT configuration FROM focus_presets WHERE id=?1 AND revision=?2",
        params![input.id,input.revision],|r|r.get::<_,String>(0)).optional().map_err(storage)?
        .ok_or_else(||err(StatusCode::CONFLICT,"preset_changed"))?;
    let preset:Preset=serde_json::from_str(&encoded).map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_preset"))?;
    let focus:features::StartFocus=serde_json::from_value(json!({"goal_id":null,"task_id":null,"cycle":null,
        "learning":null,"profile_id":preset.profile_id,"minutes":preset.minutes,"difficulty":preset.difficulty,
        "block_domains":preset.block_domains,"native_apps":preset.native_apps,
        "minimize_selected_apps":preset.minimize_selected_apps}))
        .map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_preset"))?;
    let result=features::start_focus_in(&tx,focus,now_ms())?;
    tx.commit().map_err(storage)?;
    Ok(Json(result))
}
