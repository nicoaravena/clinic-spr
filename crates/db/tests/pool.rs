//! Opening the connection pool.

use clinic_db::pool::connect;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

#[tokio::test]
async fn a_server_that_isnt_there_is_a_storage_error() {
    // Nothing listens on port 1. sqlx retries a refused connection until the pool's acquire
    // timeout (5 seconds) runs out, so this test takes that long.
    let error = connect("postgres://clinic:clinic@127.0.0.1:1/clinic")
        .await
        .unwrap_err();

    assert_eq!(error.to_string(), "storage failed");
}

#[tokio::test]
async fn a_url_that_isnt_postgres_is_a_storage_error() {
    assert!(connect("not a url").await.is_err());
}

#[sqlx::test(migrations = false)]
async fn connects_to_a_running_server(_: PgPoolOptions, options: PgConnectOptions) {
    let pool = connect(options.to_url_lossy().as_str()).await.unwrap();

    let one: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(one, 1);
}
