use actix_web::{
    web, App, HttpResponse, HttpServer,
    Responder, get, post, HttpRequest, http::header::{HeaderName}
};
use regex::Regex;

#[get("/{file}")]
async fn index(req: HttpRequest) -> impl Responder {
    let server_err = "Internal Server Error \r\n";
    let bad_req = "Please pass in a valid Host header! \r\n";

    let re_comp_fail = "Unexpected error while compiling a regex!";

    let root_domain = std::env::var("ROOT_DOMAIN").expect("Environmental variabble ROOT_DOMAIN must be defined!");
    let re_domain = Regex::new(&format!(r#"((?P<username>\w*)\.)?((?P<repo>\w*)\.)?{}"#, root_domain)).expect(re_comp_fail);

    let file = match std::fs::read_to_string("./templates/index.html") {
        Ok(val) => val,
        Err(_) => String::from("")
    };

    if file.len() == 0 {
        return HttpResponse::InternalServerError().body(server_err)
    }

    let host = req.headers().get("Host");

    match host {
        Some(val) => {
            let val = val.to_str().unwrap_or_else(|_| "");

            if val.len() == 0 {
                return HttpResponse::BadRequest().body(bad_req)
            }

            let dom_caps = re_domain.captures(&val);

            match dom_caps {
                Some(dom_caps) => {
                    let username = &dom_caps.name("username").map_or("", |m| m.as_str());
                    let repo = &dom_caps.name("repo").map_or("", |m| m.as_str());

                    HttpResponse::Ok().body(format!("username: {username}\nrepo: {repo}\n"))
                },
                _ => HttpResponse::Ok().body(file)
            }
        },
        None => HttpResponse::BadRequest().body(bad_req)
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