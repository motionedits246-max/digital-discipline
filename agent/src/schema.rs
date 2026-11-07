//! One additive migration sequence, after the preserved v1–v8 migrations.
use rusqlite::Connection;
pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 let version:i32=db.query_row("PRAGMA user_version",[],|r|r.get(0))?;
 if version>12{return Err(rusqlite::Error::InvalidQuery);}
 if version<9{
  let tx=db.unchecked_transaction()?;
  crate::classification_store::migrate(&tx)?;
  tx.execute_batch("CREATE INDEX IF NOT EXISTS idx_sessions_end ON sessions(end_ms);
   CREATE INDEX IF NOT EXISTS idx_raw_events_time ON raw_events(occurred_ms);
   INSERT OR IGNORE INTO settings(key,value) VALUES('classification_auto_review','true');
   INSERT OR IGNORE INTO settings(key,value) VALUES('classification_daily_limit','20');
   PRAGMA user_version=9;")?;
  tx.commit()?;
 }
 if version<10{let tx=db.unchecked_transaction()?;crate::tracking::migrate(&tx)?;tx.execute_batch("PRAGMA user_version=10;")?;tx.commit()?;}
 if version<11{let tx=db.unchecked_transaction()?;crate::data_lifecycle::migrate(&tx)?;tx.execute_batch("PRAGMA user_version=11;")?;tx.commit()?;}
 if version<12{let tx=db.unchecked_transaction()?;crate::focus_workflow::migrate(&tx)?;crate::policy_exceptions::migrate(&tx)?;tx.execute_batch("PRAGMA user_version=12;")?;tx.commit()?;}
 Ok(())
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn bootstrap_is_repeatable_and_preserves_owner_data(){
  let d=Connection::open_in_memory().unwrap();crate::initialize_schema(&d).unwrap();d.execute("INSERT INTO category_overrides VALUES('app:editor.exe','Development',42)",[]).unwrap();crate::initialize_schema(&d).unwrap();
  assert_eq!(d.query_row("PRAGMA user_version",[],|r|r.get::<_,i32>(0)).unwrap(),12);assert_eq!(d.query_row("SELECT COUNT(*) FROM category_overrides",[],|r|r.get::<_,i64>(0)).unwrap(),1);
 }
 #[test]fn future_schema_is_rejected_before_modifying_its_tables(){
  let d=Connection::open_in_memory().unwrap();d.execute_batch("PRAGMA user_version=999;").unwrap();assert!(crate::initialize_schema(&d).is_err());assert_eq!(d.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
 }
}
