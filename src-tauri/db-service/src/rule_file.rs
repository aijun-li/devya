use entity::rule_file::{self, Column, Entity as RuleFile, Model};
use sea_orm::{
    prelude::DateTimeUtc, sea_query::OnConflict, ActiveValue::Set, DbConn, DbErr, DeleteResult,
    EntityTrait, FromQueryResult, InsertResult, IntoActiveModel, Iterable, QuerySelect,
};
use serde::Serialize;

pub struct RuleFileQuery;

#[derive(FromQueryResult, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFileNoContent {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub is_dir: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl RuleFileQuery {
    pub async fn find_all(db: &DbConn) -> Result<Vec<RuleFileNoContent>, DbErr> {
        RuleFile::find()
            .select_only()
            .columns(Column::iter().filter(|col| match col {
                Column::Content => false,
                _ => true,
            }))
            .into_model()
            .all(db)
            .await
    }

    pub async fn find_content_by_id(db: &DbConn, id: i32) -> Result<String, DbErr> {
        let model = RuleFile::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Invalid ID".into()))?;

        if model.is_dir {
            Err(DbErr::Custom("Invalid ID of Dir".into()))
        } else {
            Ok(model.content.unwrap_or_else(|| "".into()))
        }
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

    pub async fn update_content_by_id(
        db: &DbConn,
        id: i32,
        content: String,
    ) -> Result<Model, DbErr> {
        let model = RuleFile::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Invalid ID".into()))?;

        if model.is_dir {
            return Err(DbErr::Custom("Invalid ID of Dir".into()));
        }

        let mut file = model.into_active_model();
        file.content = Set(Some(content));

        RuleFile::update(file).exec(db).await
    }
}
