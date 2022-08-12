use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220812_000001_create_report_table.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Report table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Report::Table)
                    .col(
                        ColumnDef::new(Report::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Report::Date).date_time().not_null())
                    .col(ColumnDef::new(Report::Project).string().not_null())
                    .col(ColumnDef::new(Report::Name).string().not_null())
                    .col(ColumnDef::new(Report::Verdict).string().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Report table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Report::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Report {
    Table,
    Id,
    Date,
    Project,
    Name,
    Verdict,
}
