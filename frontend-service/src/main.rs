use actix_web::{web, App, HttpServer, HttpResponse};
use tera::{Tera, Context};

async fn index(tera: web::Data<Tera>) -> HttpResponse {
    let mut ctx = Context::new();
    ctx.insert("username", "User");
    let rendered = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("/templates/**/*").unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
