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

async fn try_pages(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.)?{}"#, root_domain)).unwrap();

    // Valid Host header should already be validated by Guards
    let host = req.headers()
        .get("Host")
        .unwrap()
        .to_str()
        .unwrap();
    let dom_caps = re_domain.captures(&host).unwrap();

    let username = &dom_caps.name("username").map_or("", |m| m.as_str());
    let repo = &dom_caps.name("repo").map_or("pages", |m| m.as_str());
    let path = path.into_inner();

    match std::fs::read_to_string(format!("./pages/{}/{}/{}", username, repo, path)) {
        Ok(val) => HttpResponse::Ok().body(val),
        Err(_) => HttpResponse::NotFound().into()
    }
}

async fn fetch_pages(req: HttpRequest) -> impl Responder {
    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let git_domain = std::env::var("GIT_DOMAIN").expect("Environmental variabble GIT_DOMAIN must be defined!");
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
            let username = &dom_caps.name("username").unwrap().as_str();
            let repo = &dom_caps.name("repo").map_or("pages", |m| m.as_str());

            let stat_code = std::process::Command::new("stat")
                    .arg(format!("./pages/{}/{}", username, repo))
                    .status()
                    .expect("Cannot call stat!")
                    .code();

            let stat_code = match stat_code {
                Some(val) => val,
                None => return HttpResponse::InternalServerError().body(SERVER_ERR)
            };

            if stat_code != 0 {
                std::process::Command::new("git")
                    .arg("clone")
                    .arg(format!("{}/{}/{}.git", git_domain, username, repo))
                    .arg(format!("./pages/{}/{}", username, repo))
                    .env("GIT_TERMINAL_PROMPT", "0")
                    .status()
                    .expect("Cannot call git!");
            } else {
               std::process::Command::new("git") 
                .arg("fetch")
                .arg(format!("./pages/{}/{}", username, repo))
                .env("GIT_TERMINAL_PROMPT", "0")
                .status()
                .expect("Cannot call git!");
            }


            HttpResponse::Ok().into()
        },
        None => HttpResponse::InternalServerError().body(SERVER_ERR)
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
    match std::fs::read_to_string("./templates/404.html") {
        Ok(val) => HttpResponse::NotFound().body(val),
        Err(_) => {
            return HttpResponse::InternalServerError().body(SERVER_ERR)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");

    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.){}"#, root_domain)).unwrap();

    HttpServer::new(move || {
        App::new()
        .service(
            web::resource("/{any:.*}")
            .guard(guard::Get())
            .guard(guard::Host(format!("{root_domain}")))
            .to(index))
        .service(
            web::resource("/{filename:.*}")
            .guard(guard::Get())
            .guard(HostPattern(re_domain.to_owned()))
            .to(try_pages))
        .service(
            web::resource("/")
            .guard(guard::Post())
            .guard(HostPattern(re_domain.to_owned()))
            .to(fetch_pages)
        )
        .service(
            web::scope("")
            .route("", web::to(|| async { HttpResponse::BadRequest()} )))
        .default_service(
            web::route().to(not_found)
        )
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}