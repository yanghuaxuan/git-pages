use actix_web::{
    web, App, HttpResponse, HttpServer,
    Responder, get, post, HttpRequest, http::header::{HeaderName}
};
use regex::Regex;

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let host = req.headers().get("Host");
    let file = std::fs::read_to_string("./templates/index.html");
    let server_err_resp = "Internal Server Error \r\n";

    let root_domain = std::env::var("ROOT_DOMAIN").unwrap();
    let re = Regex::new(&format!(r#"(?P<subdomain>\w*)\.({})"#, root_domain)).unwrap();

    match host {
        Some(header) => {
            match header.to_str() {
                Ok(val) => { 
                    let domain = val.to_owned();
                    let caps = re.captures(&domain);

                    let subdomain = match caps {
                        Some(val) => val["subdomain"].to_owned(),
                        None => String::from("")
                    };

                    HttpResponse::Ok().body(val.to_owned()) 
                },
                Err(_) => HttpResponse::InternalServerError().body(server_err_resp)
            }
        },
        None => {
            match file {
                Ok(val) => HttpResponse::Ok().body(val),
                Err(_) => HttpResponse::InternalServerError().body(server_err_resp)
            }
        }
    }
}

#[post("/repos/{owner}/{repo}/deploy")]
async fn handle_deployment() -> impl Responder {
    HttpResponse::Ok().body("Deploying page!\n")
}

async fn not_found() -> impl Responder {
    let file = std::fs::read_to_string("./templates/404.html").unwrap();
    HttpResponse::NotFound().body(file)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(move || {
        App::new()
        .service(index)
        .service(handle_deployment)
        .default_service(
            web::route().to(not_found)
        )
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}