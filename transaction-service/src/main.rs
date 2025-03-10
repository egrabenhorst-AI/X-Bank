use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
struct Transaction {
    id: i32,
    account_id: i32,
    amount: f64,
    kind: String,
}

async fn deposit(pool: web::Data<PgPool>, tx: web::Json<Transaction>) -> HttpResponse {
    let result = sqlx::query!(
        "INSERT INTO transactions (account_id, amount, kind) VALUES ($1, $2, $3) RETURNING id",
        tx.account_id, tx.amount, tx.kind
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            sqlx::query!(
                "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
                tx.amount, tx.account_id
            )
            .execute(pool.get_ref())
            .await
            .unwrap();
            HttpResponse::Ok().finish()
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres://xbank_user:xbank_pass@postgres:5432/xbank").await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/deposit", web::post().to(deposit))
    })
    .bind("0.0.0.0:8083")?
    .run()
    .await
}
