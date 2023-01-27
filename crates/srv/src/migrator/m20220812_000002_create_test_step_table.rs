use crate::migrator::m20220812_000001_create_report_table::Report;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220812_000002_create_test_step_table.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Report table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestStep::Table)
                    .col(
                        ColumnDef::new(TestStep::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TestStep::Name).string().not_null())
                    .col(ColumnDef::new(TestStep::StepNumber).integer().not_null())
                    .col(ColumnDef::new(TestStep::Date).date_time().not_null())
                    .col(ColumnDef::new(TestStep::Verdict).string().not_null())
                    .col(ColumnDef::new(TestStep::ReportId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("result-guide-report_id")
                            .from(TestStep::Table, TestStep::ReportId)
                            .to(Report::Table, Report::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Report table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestStep::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TestStep {
    Table,
    Id,
    Name,
    StepNumber,
    Date,
    Verdict,
    ReportId,
}
