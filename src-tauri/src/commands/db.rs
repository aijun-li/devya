use db_service::{self, RuleDirMutation, RuleDirQuery};
use sea_orm::InsertResult;
use tauri::State;

use crate::state::DbState;

#[tauri::command]
pub async fn get_rule_dirs(
    db_state: State<'_, DbState>,
) -> Result<Vec<entity::rule_dir::Model>, String> {
    RuleDirQuery::find_all(&db_state.conn)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upsert_rule_dir(
    id: Option<i32>,
    name: String,
    parent_id: Option<i32>,
    db_state: State<'_, DbState>,
) -> Result<i32, String> {
    let result = RuleDirMutation::upsert(&db_state.conn, id, &name, parent_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.last_insert_id)
}
