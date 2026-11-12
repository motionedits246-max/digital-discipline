use crate::{err, features, ApiResult, AppState};
use axum::{extract::State, http::StatusCode, Json};
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

const MAX_CUSTOM: usize = 64;

fn names(db: &Connection, key: &str) -> Vec<String> {
    serde_json::from_str(&features::read_setting(db, key)).unwrap_or_default()
}

pub fn custom(db: &Connection) -> Vec<String> {
    names(db, "custom_categories")
}

pub fn distractions(db: &Connection) -> Vec<String> {
    names(db, "distraction_categories")
}

pub fn known(db: &Connection, name: &str) -> bool {
    features::CATEGORIES.contains(&name) || custom(db).iter().any(|label| label == name)
}

pub fn valid_selection(db: &Connection, selection: &[String]) -> bool {
    let custom = custom(db);
    selection.len() <= features::CATEGORIES.len() + MAX_CUSTOM
        && selection.iter().all(|label| {
            features::CATEGORIES.contains(&label.as_str()) || custom.contains(label)
        })
}

fn store(db: &Connection, key: &str, values: &[String]) -> rusqlite::Result<()> {
    let json = serde_json::to_string(values).expect("category labels serialize as strings");
    db.execute(
        "INSERT INTO settings(key,value) VALUES(?1,?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, json],
    )?;
    Ok(())
}

fn storage(_: rusqlite::Error) -> (StatusCode, String) {
    err(StatusCode::INTERNAL_SERVER_ERROR, "label_storage_failed")
}

fn create(db: &Connection, input: &str, distraction: bool) -> Result<String, (StatusCode, String)> {
    let name = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if input.len() > 240 || input.chars().any(char::is_control)
        || name.is_empty() || name.chars().count() > 40
    {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_category_label"));
    }

    let tx = db.unchecked_transaction().map_err(storage)?;
    let mut labels = custom(&tx);
    let folded = name.to_lowercase();
    if features::CATEGORIES.iter().any(|label| label.to_lowercase() == folded)
        || labels.iter().any(|label| label.to_lowercase() == folded)
    {
        return Err(err(StatusCode::CONFLICT, "category_label_already_exists"));
    }
    if labels.len() >= MAX_CUSTOM {
        return Err(err(StatusCode::BAD_REQUEST, "custom_category_limit_reached"));
    }

    labels.push(name.clone());
    store(&tx, "custom_categories", &labels).map_err(storage)?;
    if distraction {
        let mut selected = distractions(&tx);
        if !selected.contains(&name) {
            selected.push(name.clone());
        }
        store(&tx, "distraction_categories", &selected).map_err(storage)?;
    }
    tx.commit().map_err(storage)?;
    Ok(name)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateLabel {
    name: String,
    #[serde(default)]
    distraction: bool,
}

pub async fn add(State(state): State<Arc<AppState>>, Json(input): Json<CreateLabel>) -> ApiResult {
    let db = state.db.lock().unwrap();
    let label = create(&db, &input.name, input.distraction)?;
    Ok(Json(json!({"ok":true,"label":label})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_distraction_is_available_for_assignment_and_selection() {
        let db = Connection::open_in_memory().unwrap();
        crate::initialize_schema(&db).unwrap();
        assert_eq!(create(&db, "  Video  research ", true).unwrap(), "Video research");
        assert!(known(&db, "Video research"));
        assert!(valid_selection(&db, &["Social".into(), "Video research".into()]));
        assert_eq!(distractions(&db), ["Video research"]);
        assert!(create(&db, "video research", false).is_err());
        assert!(create(&db, "development", false).is_err());
        assert!(create(&db, "bad\nlabel", false).is_err());
        assert!(!valid_selection(&db, &["Unknown category".into()]));
    }
}
