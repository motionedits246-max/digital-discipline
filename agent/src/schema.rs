//! One additive migration sequence, after the preserved v1–v8 migrations.
use rusqlite::Connection;
pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 let version:i32=db.query_row("PRAGMA user_version",[],|r|r.get(0))?;
 if version>10{return Err(rusqlite::Error::InvalidQuery);}
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
 Ok(())
}
