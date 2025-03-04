use axum::{routing::post, Router, response::Html, extract::State, http::StatusCode};
use askama::Template;
use pqc_kyber::{keypair, encapsulate, decapsulate, Keypair};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use sha2::{Sha256, Digest};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use rand::thread_rng;

// Simulated HSM (replace with real HSM integration later)
struct HSM {
    key: Vec<u8>,
}

impl HSM {
    fn new() -> Self {
        HSM { key: vec![0x42; 32] } // Static key for MVP
    }
    fn sign(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&self.key);
        hasher.finalize().to_vec()
    }
}

// App state
struct AppState {
    hsm: HSM,
    log: Mutex<VecDeque<String>>, // Immutable log simulation
    users: Mutex<Vec<User>>,      // Simulated user store
}

// Simulated user (replace with PostgreSQL later)
#[derive(Clone)]
struct User {
    id: String,         // Digital ID (e.g., public key hash)
    public_key: Vec<u8>, // Placeholder for quantum-safe public key
}

// Templates
#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    message: String,
}

#[derive(Template)]
#[template(path = "success.html")]
struct SuccessTemplate {
    message: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    digital_id: String, // User-provided digital ID
}

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: u64,
}

async fn login(
    State(state): State<std::sync::Arc<AppState>>,
    form: axum::Form<LoginRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    // Verify digital ID
    let users = state.users.lock().unwrap();
    let user = users.iter().find(|u| u.id == form.digital_id)
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid digital ID".to_string()))?;

    // Generate quantum-safe session key
    let keypair: Keypair = keypair(&mut thread_rng()).unwrap();
    let (ciphertext, shared_secret) = encapsulate(&keypair.public, &mut thread_rng()).unwrap();

    // Sign JWT with HSM
    let token_data = Claims {
        sub: user.id.clone(),
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
    };
    let token = encode(
        &Header::default(),
        &token_data,
        &EncodingKey::from_secret(&state.hsm.sign(&shared_secret)),
    ).unwrap();

    // Log action immutably
    let log_entry = format!("User {} logged in at {}", user.id, token_data.exp);
    let mut log = state.log.lock().unwrap();
    log.push_back(hex::encode(Sha256::digest(log_entry.as_bytes())));

    // Render success page
    let template = SuccessTemplate {
        message: format!("Logged in successfully. Token: {}", token),
    };
    Ok(Html(template.render().unwrap()))
}

async fn login_page(State(_state): State<std::sync::Arc<AppState>>) -> Html<String> {
    let template = LoginTemplate {
        message: "Enter your digital ID".to_string(),
    };
    Html(template.render().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize state with a dummy user
    let state = std::sync::Arc::new(AppState {
        hsm: HSM::new(),
        log: Mutex::new(VecDeque::new()),
        users: Mutex::new(vec![User {
            id: "user1-digital-id".to_string(),
            public_key: vec![0x01; 32], // Placeholder
        }]),
    });

    // Build server
    let app = Router::new()
        .route("/", axum::routing::get(login_page))
        .route("/login", post(login))
        .with_state(state);

    println!("Testing web server initialization");

    // Run server (TLS 1.3 in production)
    axum::Server::bind(&"127.0.0.1:8080".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}