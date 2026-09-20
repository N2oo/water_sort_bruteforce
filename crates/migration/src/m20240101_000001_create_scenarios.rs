use sea_orm_migration::prelude::*;

use crate::Scenarios;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Scenarios::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Scenarios::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Scenarios::Name).string().not_null())
                    .col(ColumnDef::new(Scenarios::Description).text().null())
                    .col(ColumnDef::new(Scenarios::Board).json_binary().not_null())
                    .col(
                        ColumnDef::new(Scenarios::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Scenarios::Table).to_owned()).await
    }
}
