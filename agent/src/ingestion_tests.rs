//! Exercise the receiving boundary and actual SQLite transactions, without a browser.
use super::*;

pub(super) fn state()->Arc<AppState>{
 let db=Connection::open_in_memory().unwrap();
 initialize_schema(&db).unwrap();
 Arc::new(AppState{db:Mutex::new(db),tracker:Mutex::new(Tracker{reconciler:Reconciler::default(),session_id:None}),
  browser:Mutex::new(BrowserStatus::default()),paused:AtomicBool::new(false),stopping:AtomicBool::new(false),storage_last_error_ms:AtomicI64::new(0),storage_error_count:AtomicU64::new(0),
  token:"runtime-test-token".into(),shutdown:Arc::new(tokio::sync::Notify::new())})
}
#[tokio::test]async fn classification_review_displays_manual_categories_instead_of_stale_unknown_jobs(){
 let s=state();s.db.lock().unwrap().execute_batch("INSERT INTO classifications VALUES('app:chosen.exe','awaiting_consent','Unclassified','old queue',1,1);INSERT INTO category_overrides VALUES('app:chosen.exe','Education',2);").unwrap();let report=classification::list(State(s)).await.unwrap().0;assert_eq!(report["jobs"][0]["category"],"Education");assert_eq!(report["jobs"][0]["state"],"owner_assigned");
}
fn playback(id:&str,instance:&str)->Event{
 Event{schema_version:"1.0".into(),event_id:id.into(),source:"extension".into(),occurred_at_ms:now_ms(),kind:"video.qualified_play".into(),profile_id:None,extension_id:None,
  payload:json!({"platform":"youtube_shorts","video_id":"Example123","instance_id":instance,"watched_ms":2300})}
}
fn rows(s:&AppState,table:&str)->i64{
 assert!(["raw_events","videos"].contains(&table));
 s.db.lock().unwrap().query_row(&format!("SELECT COUNT(*) FROM {table}"),[],|r|r.get(0)).unwrap()
}
#[tokio::test]async fn service_prompts_respect_pause_private_and_presentation_and_never_grant_consent(){
 let s=state();s.db.lock().unwrap().execute("INSERT INTO service_discovery(subject,first_ms,last_ms,checked) VALUES('app:unknown.exe',?1,?1,1)",params![now_ms()]).unwrap();
 s.paused.store(true,Ordering::SeqCst);assert_eq!(service_discovery::next(State(Arc::clone(&s))).await.unwrap().0["hidden"],true);s.paused.store(false,Ordering::SeqCst);
 s.db.lock().unwrap().execute("INSERT OR REPLACE INTO settings VALUES('presentation_privacy','true')",[]).unwrap();assert_eq!(service_discovery::next(State(Arc::clone(&s))).await.unwrap().0["hidden"],true);
 {let db=s.db.lock().unwrap();db.execute("UPDATE settings SET value='false' WHERE key='presentation_privacy'",[]).unwrap();db.execute("UPDATE settings SET value=?1 WHERE key='private_until_ms'",params![(now_ms()+60000).to_string()]).unwrap();}
 assert_eq!(service_discovery::next(State(Arc::clone(&s))).await.unwrap().0["hidden"],true);s.db.lock().unwrap().execute("UPDATE settings SET value='0' WHERE key='private_until_ms'",[]).unwrap();
 let prompt=service_discovery::next(State(Arc::clone(&s))).await.unwrap().0;assert_eq!(prompt["service"]["subject"],"app:unknown.exe");assert_eq!(prompt["duration_ms"],5500);assert!(service_discovery::next(State(Arc::clone(&s))).await.unwrap().0["service"].is_null());
 assert_eq!(s.db.lock().unwrap().query_row("SELECT llm_approved FROM service_discovery",[],|r|r.get::<_,i64>(0)).unwrap(),0);
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
#[tokio::test]
async fn task_focus_snapshots_profile_context_and_owner_outcome_remains_separate(){
 let s=state();let profile=Uuid::new_v4().to_string();let task=Uuid::new_v4().to_string();
 {let d=s.db.lock().unwrap();d.execute("INSERT INTO workflow_profiles VALUES(?1,'Python Study','[\"new-editor.exe\"]','[\"www.youtube.com\"]',1,1,0)",params![profile]).unwrap();d.execute("INSERT INTO workflow_tasks(id,profile_id,title,created_ms,updated_ms) VALUES(?1,?2,'Finish lesson exercises',1,1)",params![task,profile]).unwrap();}
 let payload=serde_json::from_value(json!({"goal_id":null,"minutes":25,"difficulty":"Balanced","block_domains":[],"task_id":task,"profile_id":profile,"cycle":{"work_minutes":25,"break_minutes":5,"cycles":2,"break_policy":"soften"}})).unwrap();
 let result=features::start_focus(State(s.clone()),Json(payload)).await.unwrap().0;
 {let d=s.db.lock().unwrap();d.execute("UPDATE workflow_profiles SET apps_json='[\"other.exe\"]' WHERE id=?1",params![profile]).unwrap();let f=features::active_focus(&d,now_ms()).unwrap().unwrap();assert_eq!(f["id"],result["id"]);assert_eq!(f["context"]["apps"][0],"new-editor.exe");assert_eq!(f["context"]["domains"][0],"www.youtube.com");assert_eq!(features::category_for(&d,"brave.exe",Some("www.youtube.com")),"Entertainment");assert_eq!(d.query_row("SELECT state FROM workflow_tasks WHERE id=?1",params![task],|r|r.get::<_,String>(0)).unwrap(),"active");}
 let _=features::end_focus(State(s.clone())).await.unwrap();let review=focus_workflow::reviews(State(s.clone())).await.unwrap().0;
 assert_eq!(review["reviews"][0]["context"]["intended_outcome"],"Finish lesson exercises");assert_eq!(review["reviews"][0]["coverage"],"no_observations");assert_eq!(review["reviews"][0]["task_state"],"active");
}
#[tokio::test]
async fn invalid_learning_override_rolls_back_the_whole_session_start(){
 let s=state();let payload=serde_json::from_value(json!({"goal_id":null,"minutes":25,"difficulty":"Strict","block_domains":["www.youtube.com"],"learning":{"video_ids":["Example123"],"override_rule_ids":[Uuid::new_v4().to_string()],"hide_home":true,"hide_related":true,"hide_shorts":true,"stop_autoplay":true}})).unwrap();
 assert_eq!(features::start_focus(State(s.clone()),Json(payload)).await.unwrap_err().0,StatusCode::BAD_REQUEST);
 let d=s.db.lock().unwrap();assert_eq!(d.query_row("SELECT COUNT(*) FROM focus_sessions",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}
#[tokio::test]
async fn presentation_mask_changes_notification_privacy_without_stopping_recording(){
 let s=state();let at=now_ms();let sample=|time|Observation{app:"code.exe".into(),domain:None,profile_id:None,idle:false,at_ms:time};
 observe_tick(&s,sample(at));let cmd=serde_json::from_value(json!({"enabled":true})).unwrap();let _=data_lifecycle::presentation(State(s.clone()),Json(cmd)).await.unwrap();observe_tick(&s,sample(at+2000));
 assert!(!s.paused.load(Ordering::SeqCst));let db=s.db.lock().unwrap();assert_eq!(features::read_setting(&db,"presentation_privacy"),"true");assert_eq!(db.query_row("SELECT end_ms-start_ms FROM sessions LIMIT 1",[],|r|r.get::<_,i64>(0)).unwrap(),2000);
}
