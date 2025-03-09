use axum::{
    routing::{get, post},
    Router,
    response::Html,
    extract::{State, Form},
    http::{StatusCode, header},
};
use askama::Template;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use rand::{thread_rng, Rng};

struct HSM {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl HSM {
    fn new() -> Self {
        let secret = "your-secret-key-here"; // In production, use a secure random key
        let encoding_key = EncodingKey::from_ed_der(secret.as_bytes());
        let decoding_key = DecodingKey::from_ed_der(secret.as_bytes());
        HSM {
            encoding_key,
            decoding_key,
        }
    }
}

struct AppState {
    hsm: HSM,
    users: Mutex<Vec<User>>,
    otps: Mutex<HashMap<String, (String, u64)>>, // email -> (otp, expiry)
}

#[derive(Clone)]
struct User {
    email: String,
    balance: f64,
    summary: String,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    message: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    message: String,
    step: String,
    email: String,
}

#[derive(Template)]
#[template(path = "success.html")]
struct SuccessTemplate {
    message: String,
}

#[derive(Template)]
#[template(path = "account.html")]
struct AccountTemplate {
    balance: f64,
    summary: String,
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
}

#[derive(Deserialize)]
struct VerifyRequest {
    email: String,
    otp: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

async fn register(
    State(state): State<std::sync::Arc<AppState>>,
    Form(form): Form<RegisterRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut users = state.users.lock().unwrap();
    if users.iter().any(|u| u.email == form.email) {
        return Err((StatusCode::CONFLICT, "User already exists".to_string()));
    }
    users.push(User {
        email: form.email.clone(),
        balance: 0.0,
        summary: "New account".to_string(),
    });

    let template = SuccessTemplate {
        message: "Account created successfully".to_string(),
    };
    Ok(Html(template.render().unwrap()))
}

async fn login_page(State(_state): State<std::sync::Arc<AppState>>) -> Html<String> {
    let template = LoginTemplate {
        message: "Enter your email to receive an OTP".to_string(),
        step: "request".to_string(),
        email: "".to_string(),
    };
    Html(template.render().unwrap())
}

async fn request_otp(
    State(state): State<std::sync::Arc<AppState>>,
    Form(form): Form<LoginRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let users = state.users.lock().unwrap();
    if !users.iter().any(|u| u.email == form.email) {
        return Err((StatusCode::UNAUTHORIZED, "Email not registered".to_string()));
    }

    let otp = thread_rng().gen_range(100000..999999).to_string();
    let expiry = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 300; // 5 minutes
    state.otps.lock().unwrap().insert(form.email.clone(), (otp.clone(), expiry));

    println!("OTP for {}: {}", form.email, otp);

    let template = LoginTemplate {
        message: "Enter the OTP sent to your email".to_string(),
        step: "verify".to_string(),
        email: form.email,
    };
    Ok(Html(template.render().unwrap()))
}

async fn verify_otp(
    State(state): State<std::sync::Arc<AppState>>,
    Form(form): Form<VerifyRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut otps = state.otps.lock().unwrap();
    let (stored_otp, expiry) = otps
        .get(&form.email)
        .ok_or((StatusCode::UNAUTHORIZED, "No OTP found".to_string()))?;

    if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() > *expiry {
        otps.remove(&form.email);
        return Err((StatusCode::UNAUTHORIZED, "OTP expired".to_string()));
    }

    if stored_otp != &form.otp {
        return Err((StatusCode::UNAUTHORIZED, "Invalid OTP".to_string()));
    }

    otps.remove(&form.email);

    let token_data = Claims {
        sub: form.email.clone(),
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &token_data,
        &state.hsm.encoding_key,
    ).unwrap();

    let template = SuccessTemplate {
        message: format!("Logged in successfully. Token: {}", token),
    };
    Ok(Html(template.render().unwrap()))
}

async fn account(
    State(state): State<std::sync::Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|t| t.strip_prefix("Bearer ").map(|s| s.to_string()))
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Missing or invalid Authorization header".to_string(),
        ))?;
    let claims = decode::<Claims>(&token, &state.hsm.decoding_key, &Validation::new(Algorithm::HS256))
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?
        .claims;

    let users = state.users.lock().unwrap();
    let user = users
        .iter()
        .find(|u| u.email == claims.sub)
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let template = AccountTemplate {
        balance: user.balance,
        summary: user.summary.clone(),
    };
    Ok(Html(template.render().unwrap()))
}

async fn register_page(State(_state): State<std::sync::Arc<AppState>>) -> Html<String> {
    let template = RegisterTemplate {
        message: "Register with your email".to_string(),
    };
    Html(template.render().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hsm = HSM::new();
    let state = std::sync::Arc::new(AppState {
        hsm,
        users: Mutex::new(vec![User {
            email: "test@example.com".to_string(),
            balance: 100.0,
            summary: "Initial account".to_string(),
        }]),
        otps: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/", get(register_page))
        .route("/register", post(register))
        .route("/login", get(login_page).post(request_otp))
        .route("/verify", post(verify_otp))
        .route("/account", get(account))
        .with_state(state);

    axum::Server::bind(&"127.0.0.1:8080".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}