mod entities;
mod migrator;

use entities::{prelude::*, *};

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use sea_orm::{
    ActiveValue, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
    TransactionTrait,
};
use sea_orm_migration::prelude::*;
use std::error::Error;

const DB_URL: &str = "sqlite:./sqlite.db?mode=rwc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db = Database::connect(DB_URL).await?;

    let schema_manager = SchemaManager::new(&db);

    migrator::Migrator::up(&db, None).await?;
    assert!(schema_manager.has_table("report").await?);

    let app = Router::new()
        .route("/reports", post(add_report).get(get_reports))
        .route("/reports/:id", get(get_single_report).delete(delete_report))
        .route(
            "/reports/:id/test_steps",
            post(add_teststep).get(get_teststeps_for_report),
        )
        .route("/test_steps", get(get_teststeps))
        .route("/test_steps/:id", get(get_single_teststep))
        .layer(Extension(db));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn add_report(
    Json(report): Json<report::Model>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<String, StatusCode> {
    let report_model = report::ActiveModel {
        date: ActiveValue::Set(report.date),
        project: ActiveValue::Set(report.project),
        name: ActiveValue::Set(report.name),
        verdict: ActiveValue::Set(report.verdict),
        ..Default::default()
    };

    match Report::insert(report_model).exec(&db).await {
        Ok(res) => Ok(res.last_insert_id.to_string()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_reports(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Vec<report::Model>>, StatusCode> {
    match Report::find().all(&db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_single_report(
    Path(report_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Option<report::Model>>, StatusCode> {
    match Report::find_by_id(report_id).one(&db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn add_teststep(
    Path(report_id): Path<i32>,
    Json(ts): Json<test_step::Model>,
    Extension(db): Extension<DatabaseConnection>,
) -> (StatusCode, String) {
    match Report::find_by_id(report_id).one(&db).await {
        Ok(Some(report)) => {
            let ts_model = test_step::ActiveModel {
                name: ActiveValue::Set(ts.name),
                step_number: ActiveValue::Set(ts.step_number),
                date: ActiveValue::Set(ts.date),
                verdict: ActiveValue::Set(ts.verdict),
                report_id: ActiveValue::Set(report.id),
                ..Default::default()
            };
            match TestStep::insert(ts_model).exec(&db).await {
                Ok(res) => (StatusCode::OK, res.last_insert_id.to_string()),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            format!("Could not find report with id {}!", report_id),
        ),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
    }
}

async fn get_single_teststep(
    Path(teststp_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Option<test_step::Model>>, StatusCode> {
    match TestStep::find_by_id(teststp_id).one(&db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_teststeps(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Vec<test_step::Model>>, StatusCode> {
    match TestStep::find().all(&db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_teststeps_for_report(
    Path(report_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Vec<test_step::Model>>, StatusCode> {
    match Report::find_by_id(report_id).one(&db).await {
        Ok(Some(report)) => match report.find_related(TestStep).all(&db).await {
            Ok(res) => Ok(Json(res)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_report(
    Path(report_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> (StatusCode, String) {
    match Report::find_by_id(report_id).one(&db).await {
        Ok(Some(report)) => match db
            .transaction(|txn| {
                Box::pin(async move {
                    test_step::Entity::delete_many()
                        .filter(test_step::Column::ReportId.eq(report.id))
                        .exec(txn)
                        .await?;
                    report.delete(txn).await
                })
            })
            .await
        {
            Ok(_) => (StatusCode::OK, String::new()),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            format!("Could not find report with id {}!", report_id),
        ),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
    }
}
