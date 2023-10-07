use tokio::time::{Duration, sleep};
use tokio::task::{JoinSet};
use sqlx::PgPool;
use std::sync::Arc;

use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up tracing subscriber");

    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = Arc::new(PgPool::connect(&conn_str).await?);

    let num_tasks = 100;

    let mut join_set = JoinSet::new();

    info!("create tasks");

    for _ in 0..num_tasks {
        let pool_clone = pool.clone();
        let fut = task_function(pool_clone);
        join_set.spawn(fut);
    }

    info!("wait for tasks");
    while let Some(res) = join_set.join_next().await {
        if let Err(err) = res {
            warn!("Task failed: {:?}", err);
        }
    }

    Ok(())
}

async fn task_function(pool: Arc<PgPool>) {

    // sleep(Duration::from_millis(200)).await;

    let mut transaction = pool.begin().await.expect("Failed to begin transaction");
    transaction.commit().await.expect("Failed to commit transaction");

}
