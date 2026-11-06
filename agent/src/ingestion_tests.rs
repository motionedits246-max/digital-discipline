//! Exercise the receiving boundary and actual SQLite transactions, without a browser.
use super::*;

fn state()->Arc<AppState>{
 let db=Connection::open_in_memory().unwrap();
 initialize_schema(&db).unwrap();
 Arc::new(AppState{db:Mutex::new(db),tracker:Mutex::new(Tracker{reconciler:Reconciler::default(),session_id:None}),
  browser:Mutex::new(BrowserStatus::default()),paused:AtomicBool::new(false),stopping:AtomicBool::new(false),storage_last_error_ms:AtomicI64::new(0),storage_error_count:AtomicU64::new(0),
  token:"runtime-test-token".into(),shutdown:Arc::new(tokio::sync::Notify::new())})
}
fn playback(id:&str,instance:&str)->Event{
 Event{schema_version:"1.0".into(),event_id:id.into(),source:"extension".into(),occurred_at_ms:now_ms(),kind:"video.qualified_play".into(),profile_id:None,extension_id:None,
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

#[tokio::test]
async fn graceful_shutdown_flushes_and_rejects_later_events_without_changing_pause_preference(){
 let s=state();observe_tick(&s,Observation{app:"code.exe".into(),domain:None,profile_id:None,idle:false,at_ms:1000});
 let _=shutdown_agent(State(s.clone())).await;assert!(s.stopping.load(Ordering::SeqCst));assert!(s.tracker.lock().unwrap().session_id.is_none());
 assert_eq!(event_ingest(State(s.clone()),Json(playback(&Uuid::new_v4().to_string(),&Uuid::new_v4().to_string()))).await.unwrap_err().0,StatusCode::SERVICE_UNAVAILABLE);
 assert_eq!(features::read_setting(&s.db.lock().unwrap(),"paused"),"false");
}
#[tokio::test]
async fn desktop_boundary_rejects_web_origins_and_foreign_hosts_even_with_token(){
 let s=state();let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
 let app=Router::new().route("/test",get(||async{"accepted"})).layer(middleware::from_fn_with_state(s,auth));
 let task=tokio::spawn(async move{axum::serve(listener,app).await.unwrap()});let client=reqwest::Client::new();
 for (host,origin,expected) in [("evil.example",None,403),("127.0.0.1:48761",Some("https://evil.example"),403),("127.0.0.1:48761",None,200)]{
  let mut req=client.get(format!("http://{address}/test")).header("host",host).header("authorization","Bearer runtime-test-token");if let Some(o)=origin{req=req.header("origin",o);}
  assert_eq!(req.send().await.unwrap().status().as_u16(),expected);
 }
 task.abort();
}

#[tokio::test]
async fn route_visit_and_duplicate_are_not_qualified_playback(){
 let s=state();let id=Uuid::new_v4().to_string();
 let event=||Event{schema_version:"1.0".into(),event_id:id.clone(),source:"extension".into(),occurred_at_ms:now_ms(),kind:"video.navigation".into(),profile_id:None,extension_id:None,payload:json!({"platform":"youtube_shorts","video_id":"Example123"})};
 assert_eq!(event_ingest(State(s.clone()),Json(event())).await.unwrap().0["disposition"],"accepted");
 assert_eq!(event_ingest(State(s.clone()),Json(event())).await.unwrap().0["disposition"],"duplicate");assert_eq!(rows(&s,"videos"),0);
 assert_eq!(s.db.lock().unwrap().query_row("SELECT COUNT(*) FROM video_visits",[],|r|r.get::<_,i64>(0)).unwrap(),1);
}
