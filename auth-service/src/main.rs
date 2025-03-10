use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::Utc;

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

async fn login(user: web::Json<User>) -> HttpResponse {
    if user.username == "test" && user.password == "password123" {
        let claims = Claims {
            sub: user.username.clone(),
            exp: (Utc::now().timestamp() + 3600) as usize,
        };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();
        HttpResponse::Ok().json(serde_json::json!({"token": token}))
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/login", web::post().to(login))
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
