use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
struct Account {
    id: i32,
    user_id: i32,
    balance: f64,
}

async fn get_account(pool: web::Data<PgPool>, path: web::Path<i32>) -> HttpResponse {
    let account_id = path.into_inner();
    let account = sqlx::query_as!(Account, "SELECT * FROM accounts WHERE id = $1", account_id)
        .fetch_one(pool.get_ref())
        .await;

    match account {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres://xbank_user:xbank_pass@postgres:5432/xbank").await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/account/{id}", web::get().to(get_account))
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await
}
