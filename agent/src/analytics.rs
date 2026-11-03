//! Aggregate interval math and video count queries shared by multiple endpoints.
use crate::{policy,domain};
use rusqlite::{params, Connection};
pub fn active_domain_ms(db:&Connection,domains:&[String],start:i64,end:i64)->rusqlite::Result<i64>{
    if domains.is_empty(){return Ok(0)}
    let mut stmt=db.prepare("SELECT start_ms,end_ms,domain FROM sessions WHERE idle=0 AND end_ms>?1 AND start_ms<?2")?;
    let matched=stmt.query_map(params![start,end],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<String>>(2)?)))?
      .collect::<rusqlite::Result<Vec<_>>>()?.into_iter()
      .filter(|(_,_,d)|d.as_ref().is_some_and(|host|domains.iter().any(|wanted|host==wanted||host.ends_with(&format!(".{wanted}")))))
      .map(|(a,b,_)|(a,b)).collect();
    Ok(policy::union_ms(matched,start,end))
}
pub fn qualified_count(db:&Connection,domains:&[String],start:i64, include_replays:bool)->rusqlite::Result<i64>{
    let platforms:Vec<&str>=domains.iter().filter_map(|x|match x.as_str(){"youtube.com"|"www.youtube.com"=>Some("youtube_shorts"),"instagram.com"|"www.instagram.com"=>Some("instagram_reels"),_=>None}).collect();
    if platforms.is_empty(){return Ok(0)}
    let mut stmt=db.prepare("SELECT platform,video_id FROM videos WHERE qualified=1 AND occurred_ms>=?1")?;
    let rows=stmt.query_map(params![start],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let matches=rows.into_iter().filter(|(p,_)|platforms.contains(&p.as_str())).collect::<Vec<_>>();
    if include_replays {Ok(matches.len() as i64)}else{Ok(matches.into_iter().collect::<std::collections::BTreeSet<_>>().len() as i64)}
}
pub fn valid_target(domain_name:&str,path:&Option<String>)->bool{domain::valid_domain(domain_name)&&path.as_ref().is_none_or(|p|domain::safe_path(p))}

/// Number of qualifying short-form play instances in a configured rolling interval.
pub fn rolling_play_count(db:&Connection,domains:&[String],start:i64,gap_ms:i64)->rusqlite::Result<i64>{
    let platforms:Vec<&str>=domains.iter().filter_map(|x|match x.as_str(){
        "youtube.com"|"www.youtube.com"=>Some("youtube_shorts"),
        "instagram.com"|"www.instagram.com"=>Some("instagram_reels"),_=>None}).collect();
    if platforms.is_empty(){return Ok(0);}
    let mut stmt=db.prepare("SELECT platform,occurred_ms FROM videos WHERE qualified=1 AND occurred_ms>=?1 ORDER BY occurred_ms")?;
    let observed=stmt.query_map(params![start],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let times=observed.into_iter().filter(|(p,_)|platforms.contains(&p.as_str())).map(|(_,t)|t).collect();
    Ok(streak_tail(times,gap_ms))
}
/// Only the last uninterrupted sequence counts, with a two-minute maximum gap
/// between observed qualifying plays. This is not an inference of actual attention.
fn streak_tail(mut times:Vec<i64>,max_gap:i64)->i64 {
    times.sort_unstable();
    let(mut count,mut last)=(0,None);
    for time in times {
        if !last.is_some_and(|previous|time>previous && time-previous<=max_gap) {count=0;}
        count+=1;last=Some(time);
    }
    count
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn streak_not_just_play_count() {
        assert_eq!(streak_tail(vec![1000,130_000,260_000],120_000),1);
        assert_eq!(streak_tail(vec![1000,60_000,95_000],120_000),3);
    }
}

/// Shared budgets can combine browser domains, native app basenames and assigned
/// categories. Windows foreground sessions are the single time authority, so a
/// session matching multiple targets contributes exactly once.
pub fn active_target_ms(db:&Connection,domains:&[String],apps:&[String],categories:&[String],
    excluded_domains:&[String], excluded_apps:&[String],start:i64,end:i64)->rusqlite::Result<i64>{
    let category_overrides=crate::features::load_categories(db)?;
    let mut stmt=db.prepare("SELECT start_ms,end_ms,app,domain FROM sessions WHERE idle=0 AND end_ms>?1 AND start_ms<?2")?;
    let matches=stmt.query_map(params![start,end],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?)))?
      .collect::<rusqlite::Result<Vec<_>>>()?.into_iter().filter(|(_,_,app,domain)|{
        let host=domain.as_deref().unwrap_or("");
        let includes=apps.iter().any(|v|v==app)||
            domains.iter().any(|v|host==v || host.ends_with(&format!(".{v}"))) ||
            categories.contains(&crate::features::resolved_category(&category_overrides,app,domain.as_deref()));
        let excluded=excluded_apps.iter().any(|v|v==app) || excluded_domains.iter().any(|v|host==v||host.ends_with(&format!(".{v}")));
        includes && !excluded
      }).map(|(start,end,_,_)|(start,end)).collect();
    Ok(policy::union_ms(matches,start,end))
}

#[cfg(test)]mod shared_budget_tests {
 use super::*;
 #[test]fn mixed_shared_budget_unions_foreground_intervals_and_respects_exclusions() {
  let db=Connection::open_in_memory().unwrap();
  db.execute_batch("CREATE TABLE sessions(id TEXT,start_ms INTEGER,end_ms INTEGER,app TEXT,idle INTEGER,domain TEXT);
      CREATE TABLE category_overrides(subject TEXT,category TEXT);
      CREATE TABLE classifications(subject TEXT,category TEXT,state TEXT);
      INSERT INTO sessions VALUES('a',0,20000,'cursor.exe',0,NULL);
      INSERT INTO sessions VALUES('b',20000,30000,'brave.exe',0,'www.youtube.com');
      INSERT INTO sessions VALUES('c',25000,40000,'brave.exe',0,'www.youtube.com');").unwrap();
  let all=active_target_ms(&db,&["www.youtube.com".into()],&["cursor.exe".into()],&["Development".into()],&[],&[],0,40_000).unwrap();
  assert_eq!(all,40_000); // cursor AND its category match only once
  let excluded=active_target_ms(&db,&["www.youtube.com".into()],&["cursor.exe".into()],&[],&["www.youtube.com".into()],&[],0,40_000).unwrap();
  assert_eq!(excluded,20_000);
 }
}
