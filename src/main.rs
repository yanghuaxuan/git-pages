use actix_web::{
    web, App, HttpResponse, HttpServer,
    Responder, get, post, HttpRequest, http::header::{HeaderName}
};
use regex::Regex;

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let server_err = "Internal Server Error \r\n";
    let bad_req = "Please pass in a valid Host header! \r\n";

    let file = match std::fs::read_to_string("./templates/index.html") {
        Ok(val) => val,
        Err(_) => String::from("")
    };

    if file.len() == 0 {
        return HttpResponse::InternalServerError().body(server_err)
    }

    let host = req.headers().get("Host");

    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re = Regex::new(&format!(r#"(?P<subdomain>\w*)\.({})"#, root_domain)).expect("Unexpected error while compiling a regex!");

    match host {
        Some(val) => {
            let val = val.to_str().unwrap_or_else(|_| "");

            if val.len() == 0 {
                return HttpResponse::BadRequest().body(bad_req)
            }

            let caps = re.captures(&val);

            match caps {
                Some(val) => HttpResponse::Ok().body(val["subdomain"].to_owned()),
                None => HttpResponse::Ok().body(file)
            }
        },
        None => HttpResponse::BadRequest().body(bad_req)
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