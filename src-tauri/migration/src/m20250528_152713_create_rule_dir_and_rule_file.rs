use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RuleDir::Table)
                    .if_not_exists()
                    .col(pk_auto(RuleDir::Id))
                    .col(string(RuleDir::Name))
                    .col(integer_null(RuleDir::ParentId))
                    .col(timestamp(RuleDir::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(RuleDir::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_rule_dir_parent_id")
                            .from(RuleDir::Table, RuleDir::ParentId)
                            .to(RuleDir::Table, RuleDir::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RuleFile::Table)
                    .if_not_exists()
                    .col(pk_auto(RuleFile::Id))
                    .col(string(RuleFile::Name))
                    .col(integer_null(RuleFile::ParentId))
                    .col(timestamp(RuleFile::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(RuleFile::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_rule_file_parent_id")
                            .from(RuleFile::Table, RuleFile::ParentId)
                            .to(RuleDir::Table, RuleDir::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RuleDir::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RuleFile::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum RuleDir {
    Table,
    Id,
    Name,
    ParentId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RuleFile {
    Table,
    Id,
    Name,
    ParentId,
    CreatedAt,
    UpdatedAt,
}
