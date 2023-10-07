use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::spawn;
use tokio::time::{sleep, Duration};

use tokio::sync::mpsc;

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
        .expect("can't connect to database");

    let pool = Arc::new(pool);

    let num_tasks = 2;

    let mut handles = vec![];
    for _ in 0..num_tasks {
        let (tx, mut rx) = mpsc::channel::<()>(10);

        let handle = spawn(task_function(pool.clone(), tx));

        if let Some(_) = rx.recv().await {
            handle.abort();
        }

        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(_) => {
                println!("ok");
            }
            Err(err) => {
                // println!("err: {:?}", err);
            }
        };
    }

    Ok(())
}

async fn task_function(pool: Arc<PgPool>, tx: mpsc::Sender<()>) {
    println!("start transaction");

    let mut transaction = pool.begin().await.expect("Failed to begin transaction");

    tx.send(()).await.expect("Failed to send message to main");

    // println!("sleep");
    // sleep(Duration::from_millis(0)).await;

    println!("commit");
    transaction
        .commit()
        .await
        .expect("Failed to commit transaction");
}
