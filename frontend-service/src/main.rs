use axum::{routing::get, Router, response::Html};
use maud::html;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(home_page));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3004));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn home_page() -> Html<String> {
    let markup = html! {
        html {
            head {
                title { "X-Bank" }
            }
            body {
                h1 { "Welcome to X-Bank" }
                p { "Check your balance, make transfers, or enjoy your UBI!" }
            }
        }
    };
    Html(markup.into_string())
}