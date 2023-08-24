use actix_web::{
    web, App, HttpResponse, HttpServer,
    Responder, HttpRequest, guard::{self, Guard}
};
use regex::Regex;

static SERVER_ERR: &str = "Internal Server Error \r\n";

struct HostPatternGuard {
    host_pattern: Regex
}

impl Guard for HostPatternGuard {
    fn check(&self, ctx: &guard::GuardContext<'_>) -> bool {
        let host: Option<&str> = ctx.head().headers()
            .get("Host")
            .map_or(None, |val| val.to_str().ok());
        let host = match host {
            Some(val) => val,
            None => {
                return false
            }
        };

        match self.host_pattern.captures_iter(host).next() {
            Some(_) => true,
            None => false
        }
    }
}

#[allow(non_snake_case)]
fn HostPattern(host_pattern: Regex) -> HostPatternGuard {
    HostPatternGuard { host_pattern }
}

async fn try_pages(req: HttpRequest) -> impl Responder {
    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.)?{}"#, root_domain)).unwrap();

    // Valid Host header should already be validated by Guards
    let host = req.headers()
        .get("Host")
        .unwrap()
        .to_str()
        .unwrap();

    let dom_caps = re_domain.captures(&host);

    match dom_caps {
        Some(dom_caps) => {
            let username = &dom_caps.name("username").map_or("", |m| m.as_str());
            let repo = &dom_caps.name("repo").map_or("", |m| m.as_str());

            HttpResponse::Ok().body(format!("username: {username}\nrepo: {repo}\n"))
        },
        None => HttpResponse::InternalServerError().into()
    }
}

async fn index() -> impl Responder {
    match std::fs::read_to_string("./templates/index.html") {
        Ok(val) => HttpResponse::Ok().body(val),
        Err(_) => {
            return HttpResponse::InternalServerError().body(SERVER_ERR)
        }
    }
}

async fn not_found() -> impl Responder {
    let file = std::fs::read_to_string("./templates/404.html").unwrap();
    HttpResponse::NotFound().body(file)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");

    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.){}"#, root_domain)).unwrap();

    HttpServer::new(move || {
        App::new()
        .service(
            web::scope("/")
            .guard(guard::Host(format!("{root_domain}")))
            .route("", web::to(index)))
        .service(
            web::scope("/")
            .guard(HostPattern(re_domain.to_owned()))
            .route("", web::to(try_pages)))
        .service(
            web::scope("/")
            .route("", web::to(|| async { HttpResponse::BadRequest()} )))
        .default_service(
            web::route().to(not_found)
        )
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}