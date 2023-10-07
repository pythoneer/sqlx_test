use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up tracing subscriber");

    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&conn_str)
        .await
        .expect("can't connect to the database");

    let pool = Arc::new(pool);

    let num_tasks = 2;

    let mut handlers = JoinSet::new();
    let mut aborters = JoinSet::new();

    for _ in 0..num_tasks {
        let (tx, mut rx) = mpsc::channel::<()>(10);

        let abort_handle = Arc::new(Mutex::new(handlers.spawn(task_function(pool.clone(), tx))));
        let handle_clone = abort_handle.clone();
        aborters.spawn(async move {
            if let Some(_) = rx.recv().await {
                let mut handle = handle_clone.lock().unwrap();
                handle.abort();
            }
        });
    }

    while let Some(result) = aborters.join_next().await {
        println!("aborted")
    }

    while let Some(result) = handlers.join_next().await {
        println!("handled")
    }

    Ok(())
}

async fn task_function(pool: Arc<PgPool>, tx: mpsc::Sender<()>) {
    println!("start transaction");

    let mut transaction = pool.begin().await.expect("Failed to begin the transaction");

    tx.send(()).await.expect("Failed to send a message to main");

    // println!("sleep");
    // sleep(Duration::from_millis(0)).await;

    println!("commit");
    transaction
        .commit()
        .await
        .expect("Failed to commit the transaction");
}
