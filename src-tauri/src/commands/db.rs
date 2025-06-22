use std::{cmp::Ordering, collections::HashMap};

use db_service::{self, RuleFileMutation, RuleFileNoContent, RuleFileQuery};
use serde::Serialize;
use tauri::State;

use crate::state::DbState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFileNode {
    #[serde(flatten)]
    rule_file: RuleFileNoContent,
    children: Vec<RuleFileNode>,
}

#[tauri::command]
pub async fn get_rule_files(db_state: State<'_, DbState>) -> Result<Vec<RuleFileNode>, String> {
    let all_files = RuleFileQuery::find_all(&db_state.conn)
        .await
        .map_err(|e| e.to_string())?;

    let mut root_nodes = Vec::new();
    let mut children_map: HashMap<i32, Vec<RuleFileNode>> = HashMap::new();

    for dir in all_files {
        if let Some(parent_id) = dir.parent_id {
            children_map
                .entry(parent_id)
                .or_default()
                .push(RuleFileNode {
                    rule_file: dir,
                    children: Vec::new(),
                });
        } else {
            root_nodes.push(RuleFileNode {
                rule_file: dir,
                children: Vec::new(),
            })
        }
    }

    fn sort_file(a: &RuleFileNode, b: &RuleFileNode) -> Ordering {
        if !a.rule_file.is_dir && b.rule_file.is_dir {
            Ordering::Greater
        } else if a.rule_file.is_dir && !b.rule_file.is_dir {
            Ordering::Less
        } else {
            a.rule_file.name.cmp(&b.rule_file.name)
        }
    }

    fn build_tree(node: &mut RuleFileNode, children_map: &mut HashMap<i32, Vec<RuleFileNode>>) {
        if let Some(mut children) = children_map.remove(&node.rule_file.id) {
            children.sort_by(sort_file);
            node.children = children;
            for child in &mut node.children {
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
    root_nodes.sort_by(sort_file);

    Ok(root_nodes)
}

#[tauri::command]
pub async fn upsert_rule_file(
    id: Option<i32>,
    name: String,
    is_dir: bool,
    parent_id: Option<i32>,
    db_state: State<'_, DbState>,
) -> Result<i32, String> {
    let result = RuleFileMutation::upsert(&db_state.conn, id, &name, is_dir, parent_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.last_insert_id)
}

#[tauri::command]
pub async fn delete_rule_file(id: i32, db_state: State<'_, DbState>) -> Result<(), String> {
    RuleFileMutation::delete(&db_state.conn, id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_rule_content(id: i32, db_state: State<'_, DbState>) -> Result<String, String> {
    RuleFileQuery::find_content_by_id(&db_state.conn, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_rule_content(
    id: i32,
    content: String,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    RuleFileMutation::update_content_by_id(&db_state.conn, id, content)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
