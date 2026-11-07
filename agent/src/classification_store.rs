//! Durable classification memory and bounded job claims. No provider/network work.
use rusqlite::{params,Connection,OptionalExtension};
use serde_json::{json,Value};
use crate::{classification,features};

pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS classification_memory(
  subject TEXT PRIMARY KEY,category TEXT NOT NULL,confidence REAL NOT NULL,state TEXT NOT NULL,
  reason TEXT NOT NULL,provider TEXT NOT NULL,model TEXT NOT NULL,updated_ms INTEGER NOT NULL,last_used_ms INTEGER NOT NULL);
 CREATE TABLE IF NOT EXISTS classification_job_state(
  subject TEXT PRIMARY KEY,attempts INTEGER NOT NULL DEFAULT 0,reviewed INTEGER NOT NULL DEFAULT 0,
  lease_until_ms INTEGER NOT NULL DEFAULT 0,next_attempt_ms INTEGER NOT NULL DEFAULT 0,claim_id TEXT);
 CREATE TABLE IF NOT EXISTS classification_requests(
  id TEXT PRIMARY KEY,started_ms INTEGER NOT NULL,finished_ms INTEGER,result TEXT NOT NULL,review INTEGER NOT NULL);
 CREATE INDEX IF NOT EXISTS idx_classification_requests_time ON classification_requests(started_ms);")?;
 db.execute_batch("CREATE TABLE IF NOT EXISTS classification_attempts(subject TEXT PRIMARY KEY,attempts INTEGER NOT NULL DEFAULT 0);
  INSERT OR IGNORE INTO classification_job_state(subject,attempts,lease_until_ms)
   SELECT subject,max(COALESCE((SELECT attempts FROM classification_attempts a WHERE a.subject=c.subject),0),CASE WHEN state='running' THEN 1 ELSE 0 END),
   CASE WHEN state='running' THEN updated_ms+120000 ELSE 0 END FROM classifications c;")?;
 // Seed existing accepted decisions without changing owner overrides or enabling AI.
 db.execute("INSERT OR IGNORE INTO classification_memory(subject,category,confidence,state,reason,provider,model,updated_ms,last_used_ms)
  SELECT lower(subject),category,0.65,'accepted',reason,'legacy','',updated_ms,updated_ms FROM classifications WHERE state='succeeded' AND category!='Unclassified'",[])?;
 Ok(())
}
pub fn canonical(subject:&str)->Option<String>{
 let value=subject.trim().to_ascii_lowercase();
 if classification::safe_subject(&value){Some(value)}else{None}
}
pub fn remember(db:&Connection,subject:&str,category:&str,confidence:f64,reason:&str,provider:&str,model:&str,now:i64,review:bool)->rusqlite::Result<()> {
 let Some(subject)=canonical(subject)else{return Ok(());};
 let accepted=confidence>=0.65&&category!="Unclassified";
 let exhausted=db.query_row("SELECT attempts>=3 FROM classification_job_state WHERE subject=?1",params![subject],|r|r.get::<_,bool>(0)).optional()?.unwrap_or(false);
 let review=review||exhausted;
 let final_state=if accepted{"accepted"}else if review{"unresolved"}else{"review_pending"};
 db.execute("INSERT INTO classification_memory VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8)
  ON CONFLICT(subject) DO UPDATE SET category=excluded.category,confidence=excluded.confidence,state=excluded.state,
  reason=excluded.reason,provider=excluded.provider,model=excluded.model,updated_ms=excluded.updated_ms,last_used_ms=excluded.last_used_ms",
  params![subject,category,confidence,final_state,reason,provider,model,now])?;
 db.execute("UPDATE classifications SET category=?2,state=?3,reason=?4,updated_ms=?5 WHERE lower(subject)=?1",
  params![subject,category,if accepted{"succeeded"}else if review{"unresolved"}else{"awaiting_review"},reason,now])?;
 Ok(())
}
pub fn restore_cached(db:&Connection,subject:&str,now:i64)->rusqlite::Result<bool>{
 let Some(subject)=canonical(subject)else{return Ok(false);};
 let found=db.query_row("SELECT category,state,reason FROM classification_memory WHERE subject=?1",params![subject],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional()?;
 if let Some((category,state,reason))=found{
  let state=match state.as_str(){"accepted"=>"succeeded","review_pending"=>"awaiting_review",_=>"unresolved"};
  db.execute("INSERT INTO classifications(subject,state,category,reason,created_ms,updated_ms) VALUES(?1,?2,?3,?4,?5,?5)
    ON CONFLICT(subject) DO UPDATE SET state=excluded.state,category=excluded.category,reason=excluded.reason,updated_ms=excluded.updated_ms",
    params![subject,state,category,format!("Remembered decision: {reason}"),now])?;
  db.execute("UPDATE classification_memory SET last_used_ms=?2 WHERE subject=?1",params![subject,now])?;
  return Ok(true);
 }
 Ok(false)
}
// Recheck current exclusions at the actual send boundary, including old queued jobs.
const PRIVACY_FILTER:&str="NOT EXISTS(SELECT 1 FROM json_each(COALESCE((SELECT value FROM settings WHERE key='excluded_apps'),'[]')) a WHERE lower(c.subject)='app:'||lower(a.value))
 AND NOT EXISTS(SELECT 1 FROM json_each(COALESCE((SELECT value FROM settings WHERE key='excluded_domains'),'[]')) d WHERE lower(c.subject)='domain:'||lower(d.value) OR (substr(c.subject,1,7)='domain:' AND lower(c.subject) LIKE 'domain:%.'||lower(d.value)))";
pub struct Claim{pub id:String,pub subjects:Vec<String>,pub review:bool}
pub fn claim(db:&Connection,now:i64,auto_review:bool,daily_limit:i64)->rusqlite::Result<Option<Claim>>{
 let tx=db.unchecked_transaction()?;
 tx.execute("UPDATE classifications SET state=CASE WHEN COALESCE((SELECT attempts FROM classification_job_state WHERE subject=classifications.subject),0)>=3 THEN 'unresolved' ELSE 'failed_retryable' END,
  reason='An interrupted attempt was recovered',updated_ms=?1 WHERE state='running' AND EXISTS(SELECT 1 FROM classification_job_state j WHERE j.subject=classifications.subject AND j.lease_until_ms<=?1)",params![now])?;
 let start=now.div_euclid(86_400_000)*86_400_000;
 let used:i64=tx.query_row("SELECT COUNT(*) FROM classification_requests WHERE started_ms>=?1 AND started_ms<?2",params![start,start+86_400_000],|r|r.get(0))?;
 if used>=daily_limit.clamp(1,500){return Ok(None);}
 let review=auto_review&&tx.query_row(&format!("SELECT EXISTS(SELECT 1 FROM classifications c LEFT JOIN classification_job_state j ON j.subject=c.subject WHERE c.state='awaiting_review' AND COALESCE(j.reviewed,0)=0 AND COALESCE(j.attempts,0)<3 AND COALESCE(j.next_attempt_ms,0)<=?1 AND NOT EXISTS(SELECT 1 FROM category_overrides o WHERE lower(o.subject)=lower(c.subject)) AND {PRIVACY_FILTER})"),params![now],|r|r.get::<_,bool>(0))?;
 let subjects={
  let mut q=tx.prepare(&format!("SELECT c.subject FROM classifications c LEFT JOIN classification_job_state j ON j.subject=c.subject
   WHERE COALESCE(j.attempts,0)<3 AND COALESCE(j.next_attempt_ms,0)<=?1 AND
   ((?2=1 AND c.state='awaiting_review' AND COALESCE(j.reviewed,0)=0) OR (?2=0 AND c.state IN ('awaiting_consent','failed_retryable')))
   AND NOT EXISTS(SELECT 1 FROM category_overrides o WHERE lower(o.subject)=lower(c.subject))
   AND {PRIVACY_FILTER} ORDER BY c.updated_ms,c.subject LIMIT 10"))?;
  let rows=q.query_map(params![now,review as u8],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
  rows.into_iter().filter(|s|canonical(s).as_ref()==Some(s)).collect::<Vec<_>>()
 };
 if subjects.is_empty(){return Ok(None);}
 let id=uuid::Uuid::new_v4().to_string();
 for subject in &subjects{
  tx.execute("INSERT INTO classification_job_state(subject,attempts,reviewed,lease_until_ms,next_attempt_ms,claim_id) VALUES(?1,1,?2,?3,0,?4)
   ON CONFLICT(subject) DO UPDATE SET attempts=attempts+1,reviewed=max(reviewed,excluded.reviewed),lease_until_ms=excluded.lease_until_ms,claim_id=excluded.claim_id",
   params![subject,review as u8,now+120_000,id])?;
  tx.execute("UPDATE classifications SET state='running',reason=?2,updated_ms=?3 WHERE subject=?1",params![subject,if review{"Automatic second review in progress"}else{"Batch classification in progress"},now])?;
 }
 tx.execute("INSERT INTO classification_requests VALUES(?1,?2,NULL,'in_progress',?3)",params![id,now,review as u8])?;
 tx.commit()?;Ok(Some(Claim{id,subjects,review}))
}
pub fn finish(db:&Connection,claim:&Claim,now:i64,result:&str)->rusqlite::Result<()>{
 db.execute("UPDATE classification_requests SET finished_ms=?2,result=?3 WHERE id=?1",params![claim.id,now,result])?;
 db.execute("UPDATE classification_job_state SET lease_until_ms=0,next_attempt_ms=?2 WHERE claim_id=?1",params![claim.id,now+if result=="succeeded"{60_000}else{600_000}])?;
 Ok(())
}
pub fn summary(db:&Connection,now:i64)->rusqlite::Result<Value>{
 let start=now.div_euclid(86_400_000)*86_400_000;
 let used:i64=db.query_row("SELECT COUNT(*) FROM classification_requests WHERE started_ms>=?1 AND started_ms<?2",params![start,start+86_400_000],|r|r.get(0))?;
 let remembered:i64=db.query_row("SELECT COUNT(*) FROM classification_memory WHERE state='accepted'",[],|r|r.get(0))?;
 Ok(json!({"remembered_services":remembered,"requests_today":used,"daily_request_limit":features::read_setting(db,"classification_daily_limit").parse::<i64>().unwrap_or(20),
  "automatic_review":features::read_setting(db,"classification_auto_review")!="false","reset":"UTC midnight"}))
}
#[cfg(test)]mod tests{
 use super::*;
 fn db()->Connection{let db=Connection::open_in_memory().unwrap();db.execute_batch("CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT);CREATE TABLE classifications(subject TEXT PRIMARY KEY,state TEXT,category TEXT,reason TEXT,created_ms INTEGER,updated_ms INTEGER);CREATE TABLE category_overrides(subject TEXT PRIMARY KEY,category TEXT,updated_ms INTEGER);").unwrap();migrate(&db).unwrap();db}
 #[test]fn returning_service_uses_saved_decision_without_new_ai_request(){let db=db();remember(&db,"app:Editor.EXE","Development",0.7,"Known tool","test","model",1000,false).unwrap();assert!(restore_cached(&db,"app:editor.exe",2000).unwrap());assert!(claim(&db,3000,true,20).unwrap().is_none());assert_eq!(features::category_for(&db,"editor.exe",None),"Development");}
 #[test]fn review_is_automatic_but_bounded_and_manual_choice_wins(){let db=db();db.execute("INSERT INTO classifications VALUES('app:mystery.exe','awaiting_consent','Unclassified','',1,1)",[]).unwrap();let a=claim(&db,1000,true,20).unwrap().unwrap();remember(&db,&a.subjects[0],"Utility",0.4,"Uncertain","test","m",2000,false).unwrap();finish(&db,&a,2000,"succeeded").unwrap();assert!(claim(&db,3000,true,20).unwrap().is_none());let b=claim(&db,700000,true,20).unwrap().unwrap();assert!(b.review);remember(&db,&b.subjects[0],"Utility",0.4,"Still uncertain","test","m",700100,true).unwrap();assert!(claim(&db,1400000,true,20).unwrap().is_none());db.execute("INSERT INTO category_overrides VALUES('app:mystery.exe','Education',1)",[]).unwrap();assert_eq!(features::category_for(&db,"mystery.exe",None),"Education");}
 #[test]fn exclusions_are_rechecked_before_claim_and_do_not_starve_allowed_jobs(){
  let db=db();db.execute_batch("INSERT INTO settings VALUES('excluded_apps','[\"EXCLUDED.EXE\"]');INSERT INTO settings VALUES('excluded_domains','[\"private.example.com\"]');
   INSERT INTO classifications VALUES('domain:sub.private.example.com','awaiting_review','Unclassified','',0,0);
   INSERT INTO classifications VALUES('app:manual.exe','awaiting_review','Unclassified','',0,0);
   INSERT INTO category_overrides VALUES('app:manual.exe','Education',1);
   INSERT INTO classifications VALUES('app:excluded.exe','awaiting_consent','Unclassified','',1,1);
   INSERT INTO classifications VALUES('domain:notprivate.example.com','awaiting_consent','Unclassified','',101,101);
   INSERT INTO classifications VALUES('app:allowed.exe','awaiting_consent','Unclassified','',100,100);").unwrap();
  for i in 0..15{db.execute("INSERT INTO classifications VALUES(?1,'awaiting_consent','Unclassified','',2,2)",params![format!("domain:x{i}.private.example.com")]).unwrap();}
  assert!(features::excluded(&db,"excluded.exe",None));
  let job=claim(&db,1000,true,20).unwrap().unwrap();assert!(!job.review);assert_eq!(job.subjects,vec!["app:allowed.exe","domain:notprivate.example.com"]);
  assert_eq!(db.query_row("SELECT COUNT(*) FROM classification_job_state WHERE subject NOT IN('app:allowed.exe','domain:notprivate.example.com')",[],|r|r.get::<_,i64>(0)).unwrap(),0);
  db.execute("UPDATE settings SET value='[]' WHERE key IN('excluded_apps','excluded_domains')",[]).unwrap();let next=claim(&db,2000,true,20).unwrap().unwrap();assert!(next.review);assert_eq!(next.subjects,vec!["domain:sub.private.example.com"]);
 }
 #[test]fn crash_consumes_attempt_and_request_budget(){let db=db();db.execute("INSERT INTO classifications VALUES('app:mystery.exe','awaiting_consent','Unclassified','',1,1)",[]).unwrap();assert!(claim(&db,1000,true,1).unwrap().is_some());assert!(claim(&db,200000,true,1).unwrap().is_none());assert_eq!(summary(&db,200000).unwrap()["requests_today"],1);}
}
