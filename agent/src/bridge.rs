//! Restricted, opt-in loopback transport for Chromium MV3 extensions.
//! Desktop-only master-token endpoints enroll/revoke IDs. Ordinary websites
//! cannot access bridge endpoints: extension Origin + explicit ID allowlist.
use axum::{extract::{Path,State,Request},http::{header::{ORIGIN,HOST,ACCESS_CONTROL_ALLOW_ORIGIN,ACCESS_CONTROL_ALLOW_METHODS,ACCESS_CONTROL_ALLOW_HEADERS,HeaderValue},Method,StatusCode},middleware::Next,response::{IntoResponse,Response},routing::{get,post},Json,Router};
use serde::Deserialize;
use serde_json::{json,Value};
use rusqlite::params;
use std::sync::Arc;
use crate::{AppState,ApiResult,err,now_ms};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairInput { browser:String, extension_id:String }
fn valid_extension_id(s:&str)->bool { s.len()==32 && s.bytes().all(|b|(b'a'..=b'p').contains(&b)) }
fn valid_browser(s:&str)->bool { matches!(s,"Brave"|"Chrome"|"Edge") }
fn extension_origin(s:&str)->Option<&str> {
  let id=s.strip_prefix("chrome-extension://")?;
  if valid_extension_id(id){Some(id)}else{None}
}
fn is_paired(state:&Arc<AppState>,id:&str)->bool {
  state.db.lock().unwrap().query_row("SELECT EXISTS(SELECT 1 FROM browser_pairs WHERE extension_id=?1)",params![id],|r|r.get::<_,i64>(0)).unwrap_or(0)>0
}
pub fn migrate(db:&rusqlite::Connection)->rusqlite::Result<()> {
 db.execute_batch("CREATE TABLE IF NOT EXISTS browser_pairs (extension_id TEXT PRIMARY KEY, browser TEXT NOT NULL, created_ms INTEGER NOT NULL, last_seen_ms INTEGER NOT NULL DEFAULT 0);\nCREATE TABLE IF NOT EXISTS browser_pair_requests (extension_id TEXT PRIMARY KEY, last_requested_ms INTEGER NOT NULL);")
}

pub async fn list(State(state):State<Arc<AppState>>)->ApiResult {
 let db=state.db.lock().unwrap();
 let mut stmt=db.prepare("SELECT browser,extension_id,created_ms,last_seen_ms FROM browser_pairs ORDER BY created_ms DESC")
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 let items=stmt.query_map([],|r|Ok(json!({"browser":r.get::<_,String>(0)?,"extension_id":r.get::<_,String>(1)?,"created_ms":r.get::<_,i64>(2)?,"last_seen_ms":r.get::<_,i64>(3)?})))
   .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?.collect::<rusqlite::Result<Vec<Value>>>()
   .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 let mut pending_stmt=db.prepare("SELECT extension_id,last_requested_ms FROM browser_pair_requests WHERE last_requested_ms>?1 AND extension_id NOT IN (SELECT extension_id FROM browser_pairs) ORDER BY last_requested_ms DESC LIMIT 20")
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 let pending=pending_stmt.query_map(params![now_ms()-30*60_000],|r|Ok(json!({"extension_id":r.get::<_,String>(0)?,"requested_ms":r.get::<_,i64>(1)?})))
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?.collect::<rusqlite::Result<Vec<Value>>>()
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 Ok(Json(json!({"ok":true,"pairs":items,"pending":pending})))
}
pub async fn enroll(State(state):State<Arc<AppState>>,Json(cmd):Json<PairInput>)->ApiResult {
 if !valid_browser(&cmd.browser) || !valid_extension_id(&cmd.extension_id) {return Err(err(StatusCode::BAD_REQUEST,"invalid_browser_or_extension_id"));}
 state.db.lock().unwrap().execute("INSERT INTO browser_pairs(extension_id,browser,created_ms,last_seen_ms) VALUES(?1,?2,?3,0) ON CONFLICT(extension_id) DO UPDATE SET browser=excluded.browser",
 params![cmd.extension_id,cmd.browser,now_ms()]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 let _=state.db.lock().unwrap().execute("DELETE FROM browser_pair_requests WHERE extension_id=?1",params![cmd.extension_id]);
 Ok(Json(json!({"ok":true,"message":"Browser authorized for direct local connection. Click Test connection in the extension popup. No registry or native host needed."})))
}
pub async fn revoke(Path(id):Path<String>,State(state):State<Arc<AppState>>)->ApiResult {
 if !valid_extension_id(&id){return Err(err(StatusCode::BAD_REQUEST,"invalid_extension_id"));}
 let deleted=state.db.lock().unwrap().execute("DELETE FROM browser_pairs WHERE extension_id=?1",params![id])
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 Ok(Json(json!({"ok":true,"removed":deleted>0})))
}

pub async fn cors_guard(req:Request,next:Next)->Response {
 // No broad CORS grant. Browser service workers send an Origin header
 // identifying their actual extension. HTTP sites cannot claim that Origin.
 let origin=req.headers().get(ORIGIN).and_then(|v|v.to_str().ok()).unwrap_or("").to_owned();
 if extension_origin(&origin).is_none() {return (StatusCode::FORBIDDEN,"extension_origin_required").into_response();}
 if req.headers().get(HOST).and_then(|v|v.to_str().ok())!=Some("127.0.0.1:48761") {
  return (StatusCode::FORBIDDEN,"loopback_host_required").into_response();
 }
 let mut response=if req.method()==Method::OPTIONS {StatusCode::NO_CONTENT.into_response()}else{next.run(req).await};
 response.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN,HeaderValue::from_str(&origin).unwrap());
 response.headers_mut().insert(ACCESS_CONTROL_ALLOW_METHODS,HeaderValue::from_static("GET, POST, OPTIONS"));
 response.headers_mut().insert(ACCESS_CONTROL_ALLOW_HEADERS,HeaderValue::from_static("content-type, x-dd-bridge"));
 response.headers_mut().insert("access-control-allow-private-network",HeaderValue::from_static("true"));
 response.headers_mut().insert("vary",HeaderValue::from_static("Origin"));
 response.headers_mut().insert("cache-control",HeaderValue::from_static("no-store"));
 response
}
fn request_id(headers:&axum::http::HeaderMap)->Option<&str>{headers.get(ORIGIN).and_then(|v|v.to_str().ok()).and_then(extension_origin)}
fn request_approval(state:&Arc<AppState>,id:&str){
 let _=state.db.lock().unwrap().execute("INSERT INTO browser_pair_requests(extension_id,last_requested_ms) VALUES(?1,?2) ON CONFLICT(extension_id) DO UPDATE SET last_requested_ms=excluded.last_requested_ms",
   params![id,now_ms()]);
}
async fn touch(state:&Arc<AppState>,id:&str){let _=state.db.lock().unwrap().execute("UPDATE browser_pairs SET last_seen_ms=?1 WHERE extension_id=?2",params![now_ms(),id]);}
async fn hello(State(state):State<Arc<AppState>>,headers:axum::http::HeaderMap)->ApiResult {
 let id=request_id(&headers).ok_or_else(||err(StatusCode::FORBIDDEN,"invalid_origin"))?;
 let paired=is_paired(&state,id);
 if paired{touch(&state,id).await;}else{request_approval(&state,id);}
 Ok(Json(json!({"ok":true,"paired":paired,"transport":"loopback","schema_version":"1.0"})))
}
async fn sync(State(state):State<Arc<AppState>>,headers:axum::http::HeaderMap,Json(info):Json<crate::tracking::SyncInfo>)->ApiResult {
 let id=request_id(&headers).ok_or_else(||err(StatusCode::FORBIDDEN,"invalid_origin"))?;
 if !is_paired(&state,id){request_approval(&state,id);return Err(err(StatusCode::FORBIDDEN,"extension_not_authorized_open_desktop_setup"));}
 let profile=info.profile_id.clone();
 let status={let db=state.db.lock().unwrap();let browser=db.query_row("SELECT browser FROM browser_pairs WHERE extension_id=?1",params![id],|r|r.get::<_,String>(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
 crate::tracking::sync(&db,id,&browser,info,now_ms())};
 status.map_err(|code|err(if code=="profile_revoked_authorize_in_desktop"{StatusCode::FORBIDDEN}else if code=="storage_failed"{StatusCode::INTERNAL_SERVER_ERROR}else{StatusCode::BAD_REQUEST},code))?;
 touch(&state,id).await;
 crate::extension_rules_for_profile(state,profile).await
}
async fn event(State(state):State<Arc<AppState>>,headers:axum::http::HeaderMap,Json(mut payload):Json<crate::Event>)->ApiResult {
 let id=request_id(&headers).ok_or_else(||err(StatusCode::FORBIDDEN,"invalid_origin"))?;
 if !is_paired(&state,id){return Err(err(StatusCode::FORBIDDEN,"extension_not_authorized_open_desktop_setup"));}
 if let Some(profile)=&payload.profile_id{
  let revoked=state.db.lock().unwrap().query_row("SELECT revoked FROM browser_instances WHERE extension_id=?1 AND profile_id=?2",params![id,profile],|r|r.get::<_,i64>(0)).unwrap_or(0)!=0;
  if revoked{return Err(err(StatusCode::FORBIDDEN,"profile_revoked_authorize_in_desktop"));}
 }
 payload.extension_id=Some(id.to_owned());
 touch(&state,id).await;
 crate::event_ingest(State(state),Json(payload)).await
}
pub fn routes()->Router<Arc<AppState>> {
 Router::new()
  .route("/bridge/hello",get(hello))
  .route("/bridge/sync",post(sync))
  .route("/bridge/event",post(event))
  .layer(axum::middleware::from_fn(cors_guard))
}
#[cfg(test)] mod tests{
 use super::*;
 #[test]fn origin_only_accepts_extension_id(){
  assert_eq!(extension_origin("chrome-extension://diakokddlmicacfakghilkajnojloick"),Some("diakokddlmicacfakghilkajnojloick"));
  for bad in ["https://youtube.com","null","chrome-extension://abc","chrome-extension://diakokddlmicacfakghilkajnojloick.evil","chrome-extension://AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"]{
   assert_eq!(extension_origin(bad),None);
  }
 }
 #[test]fn migrations_and_persistence(){
  let db=rusqlite::Connection::open_in_memory().unwrap(); migrate(&db).unwrap();migrate(&db).unwrap();
  db.execute("INSERT INTO browser_pairs(extension_id,browser,created_ms) VALUES(?1,'Brave',42)",params!["diakokddlmicacfakghilkajnojloick"]).unwrap();
  assert_eq!(db.query_row("SELECT browser FROM browser_pairs",[],|r|r.get::<_,String>(0)).unwrap(),"Brave");
 }
}
