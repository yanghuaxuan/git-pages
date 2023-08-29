use actix_files::NamedFile;
use actix_web::{
    guard::{self, Guard},
    http::StatusCode,
    middleware::Logger,
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use regex::Regex;

static SERVER_ERR: &str = "Internal Server Error \r\n";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");

    let root_domain =
        std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(
        r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.){}"#,
        root_domain
    ))
    .unwrap();

    // Extra environmental variable checks before starting server
    std::env::var("GIT_DOMAIN").expect("Environmental variabble GIT_DOMAIN must be defined!");

    println!("{}", root_domain);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(
                web::resource(r"/{filename:.*}")
                    .guard(guard::Host(format!("{root_domain}")))
                    .guard(guard::Get())
                    .to(index),
            )
            .service(
                web::resource("/")
                    .guard(guard::Put())
                    .guard(HostPattern(re_domain.to_owned()))
                    .to(fetch_pages),
            )
            .service(
                web::resource("/{filename:.*}")
                    .guard(HostPattern(re_domain.to_owned()))
                    .guard(guard::Get())
                    .to(try_pages),
            )
            .default_service(web::route().to(|| async { HttpResponse::BadRequest() }))
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}

struct HostPatternGuard {
    host_pattern: Regex,
}

impl Guard for HostPatternGuard {
    fn check(&self, ctx: &guard::GuardContext<'_>) -> bool {
        let host: Option<&str> = ctx
            .head()
            .headers()
            .get("Host")
            .map_or(None, |val| val.to_str().ok());
        let host = match host {
            Some(val) => val,
            None => return false,
        };

        match self.host_pattern.captures_iter(host).next() {
            Some(_) => true,
            None => false,
        }
    }
}

#[allow(non_snake_case)]
fn HostPattern(host_pattern: Regex) -> HostPatternGuard {
    HostPatternGuard { host_pattern }
}

async fn try_pages(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let root_domain =
        std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(
        r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.)?{}"#,
        root_domain
    ))
    .unwrap();

    // Valid Host header should already be validated by Guards
    let host = req.headers().get("Host").unwrap().to_str().unwrap();
    let dom_caps = re_domain.captures(&host).unwrap();

    let username = &dom_caps.name("username").map_or("", |m| m.as_str());
    let repo = &dom_caps.name("repo").map_or("pages", |m| m.as_str());

    let path = std::path::PathBuf::from(format!("./pages/{}/{}/{}", username, repo, path.as_str()));
    println!("{}\n", &path.to_str().unwrap());

    let mut file = NamedFile::open_async(&path).await;
    // Try one more time, this time attempting to serve the index file in a directory
    if file.is_err() || std::fs::metadata(&path).unwrap().is_dir() {
        let path = path.join("index.html");
        file = NamedFile::open_async(path).await;
    }
    if !file.is_err() {
        return file.unwrap().into_response(&req).customize();
    }

    NamedFile::open_async(format!("./pages/{}/{}/404.html", username, repo))
        .await
        .or(NamedFile::open_async("./templates/404.html").await)
        .map_or(
            HttpResponse::InternalServerError()
                .body(SERVER_ERR)
                .customize()
                .with_status(StatusCode::INTERNAL_SERVER_ERROR),
            |val| {
                val.into_response(&req)
                    .customize()
                    .with_status(StatusCode::NOT_FOUND)
            },
        )
}

async fn fetch_pages(req: HttpRequest) -> impl Responder {
    let root_domain =
        std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let git_domain =
        std::env::var("GIT_DOMAIN").expect("Environmental variabble GIT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(
        r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.)?{}"#,
        root_domain
    ))
    .unwrap();

    // Valid Host header should already be validated by Guards
    let host = req.headers().get("Host").unwrap().to_str().unwrap();

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
                None => return HttpResponse::InternalServerError().body(SERVER_ERR),
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
                // Get the default remote branch
                std::process::Command::new("git")
                    .arg("remote")
                    .arg("show")
                    .arg("origin")
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .expect("Cannot call git!")
                    .stdout
                    .expect("Cannot get stdout of git");

                let def_branch_out = std::process::Command::new("sed")
                    .stdin(std::process::Stdio::piped())
                    .arg("-n")
                    .arg("'s/[[:blank:]]*HEAD branch:[[:blank:]]//p'")
                    .output()
                    .expect("Cannot call sed!");

                let def_branch = String::from_utf8_lossy(&def_branch_out.stdout);

                std::process::Command::new("git")
                    .arg("switch")
                    .arg(def_branch.as_ref());

                std::process::Command::new("git")
                    .arg("-C")
                    .arg(format!("./pages/{username}/{repo}"))
                    .arg("reset")
                    .arg("--hard")
                    .arg("origin")
                    .env("GIT_TERMINAL_PROMPT", "0")
                    .status()
                    .expect("Cannot call git!");
            }

            HttpResponse::Ok().into()
        }
        None => HttpResponse::InternalServerError().body(SERVER_ERR),
    }
}

async fn index(req: HttpRequest) -> impl Responder {
    match NamedFile::open_async("./templates/index.html").await {
        Ok(val) => val.into_response(&req),
        Err(_) => return HttpResponse::InternalServerError().body(SERVER_ERR),
    }
}
