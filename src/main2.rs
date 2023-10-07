use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing_subscriber::FmtSubscriber;

use sqlx::Executor;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up tracing subscriber");

    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&conn_str)
        .await
        .expect("can't connect to database");

    let app = Router::new()
        .route(
            "/test",
            get(test), //.post(using_connection_extractor),
        )
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn test(State(pool): State<PgPool>) -> Result<String, (StatusCode, String)> {
    let mut transaction = pool.begin().await.map_err(internal_error)?;

    transaction
        .execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
        .await
        .map_err(|err| panic!("Failed to set transaction isolation level: {:#?}", err))?;

    let data: String = sqlx::query_scalar("select 'hello world from pg'")
        // .fetch_one(&mut *transaction)
        .fetch_one(&pool)
        .await
        .map_err(internal_error)?;

    transaction.commit().await.map_err(internal_error)?;

    // Ok(data)
    Err((StatusCode::INTERNAL_SERVER_ERROR, "NO".to_owned()))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
