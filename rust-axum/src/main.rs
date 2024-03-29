//! Example of application using <https://github.com/launchbadge/sqlx>
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p example-sqlx-postgres
//! ```
//!
//! Test with curl:
//!
//! ```not_rust
//! curl 127.0.0.1:8000
//! curl -X POST 127.0.0.1:8000
//! ```

use std::time::Duration;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    FromRow,
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A struct representing a Document's JSON.
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct User {
    user_id: String,
    username: String,
    email: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust-axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    // build our application with some routes
    let app = Router::new().route("/", get(read_users)).with_state(pool);

    // run it with hyper
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

async fn read_users(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(StatusCode, Json<Vec<User>>), (StatusCode, String)> {
    let users: Vec<User> = sqlx::query_as!(
        User,
        "SELECT user_id, username, email FROM \"user\" ORDER BY user_id LIMIT 100"
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::OK, Json(users)))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
