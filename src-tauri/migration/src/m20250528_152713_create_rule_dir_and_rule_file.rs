use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RuleFile::Table)
                    .if_not_exists()
                    .col(pk_auto(RuleFile::Id))
                    .col(string(RuleFile::Name))
                    .col(integer_null(RuleFile::ParentId))
                    .col(boolean(RuleFile::IsDir))
                    .col(timestamp(RuleFile::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(RuleFile::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_rule_dir_parent_id")
                            .from(RuleFile::Table, RuleFile::ParentId)
                            .to(RuleFile::Table, RuleFile::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RuleFile::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RuleFile {
    Table,
    Id,
    Name,
    ParentId,
    IsDir,
    CreatedAt,
    UpdatedAt,
}
