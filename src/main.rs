mod entities;
mod migrator;

use entities::{prelude::*, *};

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::prelude::*;
use serde_json::{json, Value};
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
        .route("/reports/:id", get(get_single_report))
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
