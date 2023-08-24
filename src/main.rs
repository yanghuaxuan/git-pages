use actix_web::{
    web, App, HttpResponse, HttpServer,
    Responder, get, post, HttpRequest, http::header::{HeaderName}
};
use regex::Regex;

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let server_err = "Internal Server Error \r\n";
    let bad_req = "Please pass in a valid Host header! \r\n";

    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.)?{}"#, root_domain)).unwrap();

    let file = match std::fs::read_to_string("./templates/index.html") {
        Ok(val) => val,
        Err(_) => {
            return HttpResponse::InternalServerError().body(server_err)
        }
    };

    let host: Option<&str> = req.headers()
        .get("Host")
        .map_or(None, |val| val.to_str().ok());
    let host = match host {
        Some(val) => val,
        None => {
            return HttpResponse::BadRequest().body(bad_req)
        }
    };

    let dom_caps = re_domain.captures(&host);

    match dom_caps {
        Some(dom_caps) => {
            let username = &dom_caps.name("username").map_or("", |m| m.as_str());
            let repo = &dom_caps.name("repo").map_or("", |m| m.as_str());

            if username.len() == 0 && repo.len() == 0 {
                return HttpResponse::Ok().body(file)
            }

            HttpResponse::Ok().body(format!("username: {username}\nrepo: {repo}\n"))
        },
        None => HttpResponse::InternalServerError().into()
    }
}

async fn not_found() -> impl Responder {
    let file = std::fs::read_to_string("./templates/404.html").unwrap();
    HttpResponse::NotFound().body(file)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");

    HttpServer::new(move || {
        App::new()
        .service(index)
        .default_service(
            web::route().to(not_found)
        )
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}