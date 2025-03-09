use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://user:password@localhost:5432/xbank").await?;
    let app = Router::new()
        .route("/login", post(login_handler))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn login_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, String> {
    let user = sqlx::query!("SELECT password_hash FROM users WHERE username = $1", payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| "Database error")?
        .ok_or("User not found")?;

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| "Invalid hash")?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid credentials")?;

    let claims = Claims { sub: payload.username, exp: 3600 };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref()))
        .map_err(|_| "Token creation failed")?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}