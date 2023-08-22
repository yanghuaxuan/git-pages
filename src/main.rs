use actix_web::{
    web, App, HttpResponse, HttpServer,
    Responder, get, post
};

#[get("/index.html")]
async fn index() -> impl Responder {
    let file = std::fs::read_to_string("./templates/index.html");

    match file {
        Ok(file) => HttpResponse::Ok().body(file),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error\n")
    }
}

#[post("/repos/{owner}/{repo}/deploy")]
async fn handle_deployment() -> impl Responder {
    HttpResponse::Ok()
}

async fn not_found() -> impl Responder {
    let file = std::fs::read_to_string("./templates/404.html").unwrap();
    HttpResponse::NotFound().body(file)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new().service(index)
        .default_service(
            web::route().to(not_found)
        )
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}