use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Serialize, serde::Deserialize)]
struct Account {
    id: i32,
    user_id: i32,
    balance: f64,
}

async fn distribute_ubi(pool: web::Data<PgPool>) -> HttpResponse {
    let ubi_amount = 1000.0;
    let accounts = sqlx::query_as!(Account, "SELECT * FROM accounts")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    for account in accounts {
        sqlx::query!(
            "INSERT INTO transactions (account_id, amount, kind) VALUES ($1, $2, 'ubi')",
            account.id, ubi_amount
        )
        .execute(pool.get_ref())
        .await
        .unwrap();

        sqlx::query!(
            "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
            ubi_amount, account.id
        )
        .execute(pool.get_ref())
        .await
        .unwrap();
    }
    HttpResponse::Ok().json(serde_json::json!({"status": "UBI distributed"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres://xbank_user:xbank_pass@postgres:5432/xbank").await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/distribute-ubi", web::post().to(distribute_ubi))
    })
    .bind("0.0.0.0:8084")?
    .run()
    .await
}
