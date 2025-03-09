use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(serde::Deserialize)]
struct DepositRequest {
    account_id: i32,
    amount: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://user:password@localhost:5432/xbank").await?;
    let app = Router::new()
        .route("/deposit", post(deposit))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3002".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn deposit(
    State(pool): State<PgPool>,
    Json(payload): Json<DepositRequest>,
) -> Result<(), String> {
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        payload.amount,
        payload.account_id
    )
    .execute(&pool)
    .await
    .map_err(|_| "Deposit failed")?;

    Ok(())
}