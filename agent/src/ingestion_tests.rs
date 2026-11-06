//! Exercise the receiving boundary and actual SQLite transactions, without a browser.
use super::*;

fn state()->Arc<AppState>{
 let db=Connection::open_in_memory().unwrap();
 db.execute_batch("CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);
  CREATE TABLE raw_events(event_id TEXT PRIMARY KEY,kind TEXT NOT NULL,occurred_ms INTEGER NOT NULL);
  CREATE TABLE videos(instance_id TEXT PRIMARY KEY,platform TEXT NOT NULL,video_id TEXT NOT NULL,watched_ms INTEGER NOT NULL,qualified INTEGER NOT NULL,occurred_ms INTEGER NOT NULL);
  CREATE TABLE sessions(id TEXT PRIMARY KEY,start_ms INTEGER,end_ms INTEGER,app TEXT,idle INTEGER,domain TEXT);
  CREATE TABLE rules(id TEXT PRIMARY KEY);").unwrap();
 features::migrate(&db).unwrap();
 Arc::new(AppState{db:Mutex::new(db),tracker:Mutex::new(Tracker{reconciler:Reconciler::default(),session_id:None}),
  browser:Mutex::new(BrowserStatus::default()),paused:AtomicBool::new(false),storage_last_error_ms:AtomicI64::new(0),storage_error_count:AtomicU64::new(0),
  token:String::new(),shutdown:Arc::new(tokio::sync::Notify::new())})
}
fn playback(id:&str,instance:&str)->Event{
 Event{schema_version:"1.0".into(),event_id:id.into(),source:"extension".into(),occurred_at_ms:now_ms(),kind:"video.qualified_play".into(),
  payload:json!({"platform":"youtube_shorts","video_id":"Example123","instance_id":instance,"watched_ms":2300})}
}
fn rows(s:&AppState,table:&str)->i64{
 assert!(["raw_events","videos"].contains(&table));
 s.db.lock().unwrap().query_row(&format!("SELECT COUNT(*) FROM {table}"),[],|r|r.get(0)).unwrap()
}
#[tokio::test]
async fn retry_after_lost_reply_is_a_duplicate_not_a_second_play(){
 let s=state();let id=Uuid::new_v4().to_string();let instance=Uuid::new_v4().to_string();
 let first=event_ingest(State(Arc::clone(&s)),Json(playback(&id,&instance))).await.unwrap().0;
 assert_eq!(first["event_id"],id);assert_eq!(first["disposition"],"accepted");
 let retry=event_ingest(State(Arc::clone(&s)),Json(playback(&id,&instance))).await.unwrap().0;
 assert_eq!(retry["event_id"],id);assert_eq!(retry["disposition"],"duplicate");
 assert_eq!(rows(&s,"raw_events"),1);assert_eq!(rows(&s,"videos"),1);
}
#[tokio::test]
async fn derived_write_failure_rolls_back_dedup_id_before_retry(){
 let s=state();let id=Uuid::new_v4().to_string();let instance=Uuid::new_v4().to_string();
 s.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_video BEFORE INSERT ON videos BEGIN SELECT RAISE(FAIL,'injected write failure'); END;").unwrap();
 let failure=event_ingest(State(Arc::clone(&s)),Json(playback(&id,&instance))).await.unwrap_err();
 assert_eq!(failure.0,StatusCode::INTERNAL_SERVER_ERROR);
 assert_eq!(rows(&s,"raw_events"),0);assert_eq!(rows(&s,"videos"),0);
 s.db.lock().unwrap().execute_batch("DROP TRIGGER fail_video;").unwrap();
 let receipt=event_ingest(State(Arc::clone(&s)),Json(playback(&id,&instance))).await.unwrap().0;
 assert_eq!(receipt["disposition"],"accepted");assert_eq!(rows(&s,"videos"),1);
}
#[tokio::test]
async fn paused_and_excluded_receipts_never_claim_recorded_playback(){
 let s=state();let instance=Uuid::new_v4().to_string();
 s.paused.store(true,Ordering::SeqCst);
 let paused=event_ingest(State(Arc::clone(&s)),Json(playback(&Uuid::new_v4().to_string(),&instance))).await.unwrap().0;
 assert_eq!(paused["disposition"],"ignored_paused");assert_eq!(rows(&s,"videos"),0);
 s.paused.store(false,Ordering::SeqCst);
 s.db.lock().unwrap().execute("UPDATE settings SET value='[\"www.youtube.com\"]' WHERE key='excluded_domains'",[]).unwrap();
 let excluded=event_ingest(State(Arc::clone(&s)),Json(playback(&Uuid::new_v4().to_string(),&instance))).await.unwrap().0;
 assert_eq!(excluded["disposition"],"ignored_excluded");assert_eq!(rows(&s,"videos"),0);
}
