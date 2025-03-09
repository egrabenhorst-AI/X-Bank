use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(serde::Deserialize)]
struct AuditLog {
    action: String,
    account_id: i32,
    amount: f64,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://user:password@localhost:5432/xbank").await?;
    let app = Router::new()
        .route("/log", post(log_action))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3003".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn log_action(
    State(pool): State<PgPool>,
    Json(log): Json<AuditLog>,
) -> Result<(), String> {
    sqlx::query!(
        "INSERT INTO audit_logs (action, account_id, amount, timestamp) VALUES ($1, $2, $3, $4)",
        log.action,
        log.account_id,
        log.amount,
        log.timestamp
    )
    .execute(&pool)
    .await
    .map_err(|_| "Audit log failed")?;
    Ok(())
}