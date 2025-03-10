use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
struct AuditLog {
    action: String,
    user_id: i32,
    timestamp: i64,
    hash: String,
}

async fn log_action(pool: web::Data<SqlitePool>, log: web::Json<AuditLog>) -> HttpResponse {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}", log.action, log.user_id, log.timestamp));
    let hash = format!("{:x}", hasher.finalize());

    sqlx::query!(
        "INSERT INTO audit_logs (action, user_id, timestamp, hash) VALUES ($1, $2, $3, $4)",
        log.action, log.user_id, log.timestamp, hash
    )
    .execute(pool.get_ref())
    .await
    .unwrap();

    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePool::connect("sqlite://audit.db").await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/log", web::post().to(log_action))
    })
    .bind("0.0.0.0:8085")?
    .run()
    .await
}
