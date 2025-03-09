use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(serde::Serialize)]
struct Account {
    id: i32,
    balance: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://user:password@localhost:5432/xbank").await?;
    let app = Router::new()
        .route("/account/:id", get(get_account))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3001".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn get_account(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Account>, String> {
    let account = sqlx::query_as!(Account, "SELECT id, balance FROM accounts WHERE id = $1", id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| "Database error")?
        .ok_or("Account not found")?;

    Ok(Json(account))
}