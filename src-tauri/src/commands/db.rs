use std::collections::HashMap;

use db_service::{self, RuleDirMutation, RuleDirQuery};
use serde::Serialize;
use tauri::State;

use crate::state::DbState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDirNode {
    #[serde(flatten)]
    rule_dir: entity::rule_dir::Model,
    dirs: Vec<RuleDirNode>,
}

#[tauri::command]
pub async fn get_rule_dirs(db_state: State<'_, DbState>) -> Result<Vec<RuleDirNode>, String> {
    let all_dirs = RuleDirQuery::find_all(&db_state.conn)
        .await
        .map_err(|e| e.to_string())?;

    let mut root_nodes = Vec::new();
    let mut children_map: HashMap<i32, Vec<RuleDirNode>> = HashMap::new();

    for dir in all_dirs {
        if let Some(parent_id) = dir.parent_id {
            children_map
                .entry(parent_id)
                .or_default()
                .push(RuleDirNode {
                    rule_dir: dir,
                    dirs: Vec::new(),
                });
        } else {
            root_nodes.push(RuleDirNode {
                rule_dir: dir,
                dirs: Vec::new(),
            })
        }
    }

    fn build_tree(node: &mut RuleDirNode, children_map: &mut HashMap<i32, Vec<RuleDirNode>>) {
        if let Some(mut children) = children_map.remove(&node.rule_dir.id) {
            children.sort_by(|a, b| a.rule_dir.name.cmp(&b.rule_dir.name));
            node.dirs = children;
            for child in &mut node.dirs {
                build_tree(child, children_map);
            }
        }
    }

    let mut root_nodes: Vec<_> = root_nodes
        .into_iter()
        .map(|mut root| {
            build_tree(&mut root, &mut children_map);
            root
        })
        .collect();
    root_nodes.sort_by(|a, b| a.rule_dir.name.cmp(&b.rule_dir.name));

    Ok(root_nodes)
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

#[tauri::command]
pub async fn delete_rule_dir(id: i32, db_state: State<'_, DbState>) -> Result<(), String> {
    RuleDirMutation::delete(&db_state.conn, id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
