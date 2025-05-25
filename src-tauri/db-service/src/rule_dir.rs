use entity::{rule_dir, rule_dir::Column, rule_dir::Entity as RuleDir};
use sea_orm::{sea_query::OnConflict, ActiveValue::Set, DbConn, DbErr, EntityTrait, InsertResult};

pub struct RuleDirQuery;

impl RuleDirQuery {
    pub async fn find_all(db: &DbConn) -> Result<Vec<rule_dir::Model>, DbErr> {
        RuleDir::find().all(db).await
    }
}

pub struct RuleDirMutation;

impl RuleDirMutation {
    pub async fn upsert(
        db: &DbConn,
        id: Option<i32>,
        name: &str,
        parent_id: Option<i32>,
    ) -> Result<InsertResult<rule_dir::ActiveModel>, DbErr> {
        let new_rule_dir = rule_dir::ActiveModel {
            id: match id {
                Some(id) => Set(id),
                None => Default::default(),
            },
            name: Set(name.to_string()),
            parent_id: Set(parent_id),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        RuleDir::insert(new_rule_dir)
            .on_conflict(
                OnConflict::column(Column::Id)
                    .update_columns([Column::Name, Column::ParentId])
                    .to_owned(),
            )
            .exec(db)
            .await
    }
}
