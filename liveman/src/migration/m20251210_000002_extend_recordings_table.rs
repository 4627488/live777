use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Recordings::Table)
                    .add_column(ColumnDef::new(Recordings::StartTime).big_integer().null())
                    .add_column(ColumnDef::new(Recordings::Duration).double().null())
                    .add_column(ColumnDef::new(Recordings::Meta).json_binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Recordings::Table)
                    .drop_column(Recordings::Meta)
                    .drop_column(Recordings::Duration)
                    .drop_column(Recordings::StartTime)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Recordings {
    Table,
    StartTime,
    Duration,
    Meta,
}
