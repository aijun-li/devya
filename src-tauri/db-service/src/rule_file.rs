use entity::{rule_file, rule_file::Column, rule_file::Entity as RuleFile};
use sea_orm::{
    sea_query::OnConflict, ActiveValue::Set, DbConn, DbErr, DeleteResult, EntityTrait, InsertResult,
};

pub struct RuleFileQuery;

impl RuleFileQuery {
    pub async fn find_all(db: &DbConn) -> Result<Vec<rule_file::Model>, DbErr> {
        RuleFile::find().all(db).await
    }
}

pub struct RuleFileMutation;

impl RuleFileMutation {
    pub async fn upsert(
        db: &DbConn,
        id: Option<i32>,
        name: &str,
        is_dir: bool,
        parent_id: Option<i32>,
    ) -> Result<InsertResult<rule_file::ActiveModel>, DbErr> {
        let new_rule_file = rule_file::ActiveModel {
            id: match id {
                Some(id) => Set(id),
                None => Default::default(),
            },
            name: Set(name.to_string()),
            is_dir: Set(is_dir),
            parent_id: Set(parent_id),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        RuleFile::insert(new_rule_file)
            .on_conflict(
                OnConflict::column(Column::Id)
                    .update_columns([Column::Name, Column::ParentId])
                    .to_owned(),
            )
            .exec(db)
            .await
    }

    pub async fn delete(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        RuleFile::delete_by_id(id).exec(db).await
    }
}
