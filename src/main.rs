mod entities;
mod migrator;

use entities::{prelude::*, *};

use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::prelude::*;
use serde_json::{json, Value};
use std::error::Error;

const DB_URL: &str = "sqlite:./sqlite.db?mode=rwc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db = Database::connect(DB_URL).await?;

    let schema_manager = SchemaManager::new(&db);

    migrator::Migrator::refresh(&db).await?;
    assert!(schema_manager.has_table("report").await?);

    let app = Router::new()
        .route("/reports", post(add_report))
        .layer(Extension(db));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn add_report(
    Json(report): Json<report::Model>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Value>, StatusCode> {
    let report_model = report::ActiveModel {
        date: ActiveValue::Set(report.date),
        project: ActiveValue::Set(report.project),
        name: ActiveValue::Set(report.name),
        verdict: ActiveValue::Set(report.verdict),
        ..Default::default()
    };

    match Report::insert(report_model).exec(&db).await {
        Ok(res) => Ok(Json(json!({ "id": res.last_insert_id}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
