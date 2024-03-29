mod error;
mod migrator;

use entities::{prelude::*, *};

use crate::error::{RgError, WithStatusCode};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use eyre::eyre;
use sea_orm::{
    ActiveValue, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
    TransactionTrait,
};
use sea_orm_migration::prelude::*;
use std::error::Error;

const DB_URL: &str = "sqlite:./sqlite.db?mode=rwc";

fn init_tracing() {
    tracing_subscriber::fmt().with_test_writer().init();
}

fn app(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/reports", post(add_report).get(get_reports))
        .route("/reports/:id", get(get_single_report).delete(delete_report))
        .route(
            "/reports/:id/test_steps",
            post(add_teststep).get(get_teststeps_for_report),
        )
        .route("/test_steps", get(get_teststeps))
        .route("/test_steps/:id", get(get_single_teststep))
        .with_state(db)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    let db = Database::connect(DB_URL).await?;

    let schema_manager = SchemaManager::new(&db);

    migrator::Migrator::up(&db, None).await?;
    assert!(schema_manager.has_table("report").await?);

    let app = app(db);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn add_report(
    State(db): State<DatabaseConnection>,
    report: String,
) -> Result<String, RgError> {
    let report: report::Model =
        serde_json::from_str(&report).with_status_code(StatusCode::BAD_REQUEST)?;

    let report_model = report::ActiveModel {
        date: ActiveValue::Set(report.date),
        project: ActiveValue::Set(report.project),
        name: ActiveValue::Set(report.name),
        verdict: ActiveValue::Set(report.verdict),
        ..Default::default()
    };

    let res = Report::insert(report_model).exec(&db).await?;
    Ok(res.last_insert_id.to_string())
}

async fn get_reports(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<report::Model>>, RgError> {
    Ok(Json(
        Report::find()
            .all(&db)
            .await
            .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn get_single_report(
    Path(report_id): Path<i32>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<report::Model>, RgError> {
    match Report::find_by_id(report_id).one(&db).await {
        Ok(None) => Err(eyre!("Could not find report")).with_status_code(StatusCode::NOT_FOUND),
        Ok(Some(x)) => Ok(Json(x)),
        Err(e) => Err(RgError::from(e).with_status_code(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

async fn add_teststep(
    Path(report_id): Path<i32>,
    State(db): State<DatabaseConnection>,
    Json(ts): Json<test_step::Model>,
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
    State(db): State<DatabaseConnection>,
) -> Result<Json<Option<test_step::Model>>, StatusCode> {
    match TestStep::find_by_id(teststp_id).one(&db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_teststeps(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<test_step::Model>>, StatusCode> {
    match TestStep::find().all(&db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_teststeps_for_report(
    Path(report_id): Path<i32>,
    State(db): State<DatabaseConnection>,
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
    State(db): State<DatabaseConnection>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http;
    use axum::http::Request;
    use hyper::Body;
    use serde_json::json;
    use temp_file::TempFile;
    use tower::{Service, ServiceExt};

    #[tokio::test]
    async fn test_error_cases_for_add_report_with_wrong_format() {
        let (db, _tmp_file) = setup_empty_temp_database().await;
        let mut app = app(db);

        // Add report
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/reports")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_vec(&json!(
                        {"date": "today",
                        "project": "TestProjekt",
                        "name": "TestReport",
                        "verdict": "PASSED"}))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(
            body,
            "input contains invalid characters at line 1 column 15"
        );
    }

    async fn setup_empty_temp_database() -> (DatabaseConnection, TempFile) {
        let tmp_file = temp_file::empty();

        let db = Database::connect(format!(
            "sqlite:{}?mode=rwc",
            tmp_file.path().to_str().unwrap()
        ))
        .await
        .unwrap();
        migrator::Migrator::fresh(&db).await.unwrap();
        (db, tmp_file)
    }

    #[tokio::test]
    async fn test_report() {
        let (db, _tmp_file) = setup_empty_temp_database().await;
        let mut app = app(db);

        // Add report
        app.call(
            Request::builder()
                .method(http::Method::POST)
                .uri("/reports")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!(
                        {"date": "2014-11-28T21:00:09+09:00",
                        "project": "TestProjekt",
                        "name": "TestReport",
                        "verdict": "PASSED"}))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

        // Get report
        let response = app
            .ready()
            .await
            .unwrap()
            .call(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/reports/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            // TODO: Find the reason why the returned timestamp is in UTC
            json!(
                {
                    "date":
                    "2014-11-28T12:00:09+00:00",
                    "id": 1,
                    "project": "TestProjekt",
                    "name": "TestReport",
                    "verdict": "PASSED"
            })
        );

        // Delete report
        let response = app
            .ready()
            .await
            .unwrap()
            .call(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/reports/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body, "");

        let response = app
            .ready()
            .await
            .unwrap()
            .call(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/reports/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body, "Could not find report with id 1!");

        // Check that report got deleted
        let response = app
            .ready()
            .await
            .unwrap()
            .call(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/reports/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body, "Could not find report");
    }
}
