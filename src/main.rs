use axum::{
    routing::{get, post}, Router, response::Html, extract::{State, Form}, http::{StatusCode, header}
};
use askama::Template;
use pqc_kyber::{keypair as kyber_keypair, encapsulate, Keypair as KyberKeypair};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use sha2::{Sha256, Digest};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use rand::{thread_rng, Rng};
use ed25519_dalek::{Keypair, SigningKey, VerifyingKey, Signature, Verifier};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::convert::TryInto;

// Simulated HSM structure for JWT signing
struct HSM {
    encoding_key: EncodingKey,
}

impl HSM {
    fn new() -> (Self, DecodingKey) {
        let mut rng = thread_rng();
        let keypair = Keypair::generate(&mut rng);
        let signing_key = keypair.secret;
        let private_key = signing_key.to_bytes();
        let verifying_key = keypair.public;
        let public_key = verifying_key.to_bytes();
        let encoding_key = EncodingKey::from_ed_der(&private_key);
        let decoding_key = DecodingKey::from_ed_der(&public_key);
        (HSM { encoding_key }, decoding_key)
    }

    fn sign(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

// Application state
struct AppState {
    hsm: HSM,
    decoding_key: DecodingKey,
    log: Mutex<VecDeque<String>>, // Immutable log for transparency
    users: Mutex<Vec<User>>,
}

// User data structure
#[derive(Clone)]
struct User {
    id: String, // Base64-encoded Ed25519 public key
    balance: f64,
    summary: String,
}

// Templates for SSR
#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    message: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    message: String,
    nonce: String,
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

// Request and JWT claims structs
#[derive(Deserialize)]
struct RegisterRequest {
    digital_id: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    digital_id: String,
    signature: String,
    nonce: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

// Registration handler
async fn register(
    State(state): State<std::sync::Arc<AppState>>,
    Form(form): Form<RegisterRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let public_key_bytes = STANDARD.decode(&form.digital_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid digital ID encoding".to_string()))?;
    let public_key_array: [u8; 32] = public_key_bytes.try_into()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid public key length".to_string()))?;
    VerifyingKey::from_bytes(&public_key_array)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid public key".to_string()))?;

    let mut users = state.users.lock().unwrap();
    if users.iter().any(|u| u.id == form.digital_id) {
        return Err((StatusCode::CONFLICT, "User already exists".to_string()));
    }
    users.push(User {
        id: form.digital_id.clone(),
        balance: 0.0,
        summary: "New account".to_string(),
    });

    let log_entry = format!(
        "User {} registered at {}",
        form.digital_id,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    state.log.lock().unwrap().push_back(hex::encode(Sha256::digest(log_entry.as_bytes())));

    let template = SuccessTemplate {
        message: "Account created successfully".to_string(),
    };
    Ok(Html(template.render().unwrap()))
}

// Login handler
async fn login(
    State(state): State<std::sync::Arc<AppState>>,
    Form(form): Form<LoginRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let users = state.users.lock().unwrap();
    let user = users.iter().find(|u| u.id == form.digital_id)
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid digital ID".to_string()))?;

    let public_key_bytes = STANDARD.decode(&user.id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid digital ID encoding".to_string()))?;
    let public_key_array: [u8; 32] = public_key_bytes.try_into()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid public key length".to_string()))?;
    let public_key = VerifyingKey::from_bytes(&public_key_array)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid public key".to_string()))?;

    let signature_bytes = STANDARD.decode(&form.signature)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signature encoding".to_string()))?;
    let signature = Signature::try_from(&signature_bytes[..])
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signature format".to_string()))?;

    public_key.verify(form.nonce.as_bytes(), &signature)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Signature verification failed".to_string()))?;

    let kyber_keypair: KyberKeypair = kyber_keypair(&mut thread_rng()).unwrap();
    let (_ciphertext, _shared_secret) = encapsulate(&kyber_keypair.public, &mut thread_rng()).unwrap();

    let token_data = Claims {
        sub: user.id.clone(),
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
    };
    let token = encode(
        &Header::new(Algorithm::EdDSA),
        &token_data,
        &state.hsm.encoding_key,
    ).unwrap();

    let log_entry = format!("User {} logged in at {}", user.id, token_data.exp);
    state.log.lock().unwrap().push_back(hex::encode(Sha256::digest(log_entry.as_bytes())));

    let template = SuccessTemplate {
        message: format!("Logged in successfully. Token: {}", token),
    };
    Ok(Html(template.render().unwrap()))
}

// Account viewing handler
async fn account(
    State(state): State<std::sync::Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let token = headers.get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|t| t.strip_prefix("Bearer ").map(|s| s.to_string()))
        .ok_or((StatusCode::BAD_REQUEST, "Missing or invalid Authorization header".to_string()))?;
    let claims = decode::<Claims>(
        &token,
        &state.decoding_key,
        &Validation::new(Algorithm::EdDSA),
    ).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?.claims;

    let users = state.users.lock().unwrap();
    let user = users.iter().find(|u| u.id == claims.sub)
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let log_entry = format!(
        "User {} viewed account at {}",
        user.id,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    state.log.lock().unwrap().push_back(hex::encode(Sha256::digest(log_entry.as_bytes())));

    let template = AccountTemplate {
        balance: user.balance,
        summary: user.summary.clone(),
    };
    Ok(Html(template.render().unwrap()))
}

// Page rendering handlers
async fn register_page(State(_state): State<std::sync::Arc<AppState>>) -> Html<String> {
    let template = RegisterTemplate {
        message: "Create a new account with your digital ID (base64-encoded public key)".to_string(),
    };
    Html(template.render().unwrap())
}

async fn login_page(State(_state): State<std::sync::Arc<AppState>>) -> Html<String> {
    let nonce = STANDARD.encode(thread_rng().gen::<[u8; 32]>());
    let template = LoginTemplate {
        message: "Enter your digital ID and signature".to_string(),
        nonce,
    };
    Html(template.render().unwrap())
}

// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (hsm, decoding_key) = HSM::new();

    // Create a dummy user for testing
    let mut rng = thread_rng();
    let keypair = Keypair::generate(&mut rng);
    let signing_key = keypair.secret;
    let verifying_key = keypair.public;
    let id = STANDARD.encode(verifying_key.to_bytes());
    let state = std::sync::Arc::new(AppState {
        hsm,
        decoding_key,
        log: Mutex::new(VecDeque::new()),
        users: Mutex::new(vec![User {
            id,
            balance: 100.0,
            summary: "Initial account".to_string(),
        }]),
    });

    let app = Router::new()
        .route("/", get(register_page))
        .route("/register", post(register))
        .route("/login", get(login_page).post(login))
        .route("/account", get(account))
        .with_state(state);

    axum::Server::bind(&"127.0.0.1:8080".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}