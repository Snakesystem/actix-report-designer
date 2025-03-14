use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{ cookie::{time::Duration, Cookie, Key, SameSite}, get, middleware::{self}, post, web::{self, route}, App, HttpMessage as _, HttpRequest, HttpResponse, HttpServer, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use crate::backend::{context::{connection::create_pool, jwt_session::create_jwt}, services::generic_services::{ActionResult, GenericService, LoginRequest, WebUser}};

use super::services::generic_services;

#[get("/")]
async fn health_check() -> String {
    format!("Snakesystem Report Designer")
}

#[post("/login")]
async fn login(req: HttpRequest, pool: web::Data<Pool<ConnectionManager>>, request: web::Json<LoginRequest>) -> impl Responder {

    let result: ActionResult<WebUser, _> = GenericService::login(pool, request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => {
            if let Some(user) = &response.data {
                // ✅ Buat token JWT
                match create_jwt(user.clone()) {
                    Ok(token) => {
                        Identity::login(&req.extensions(), token.clone()).unwrap(); // ✅ Simpan sesi

                        // ✅ Simpan token dalam cookie
                        let cookie = Cookie::build("token", token.clone())
                            .path("/")
                            .http_only(true)
                            .same_site(SameSite::Strict)
                            .secure(false) // Ubah ke `true` jika pakai HTTPS
                            .finish();

                        return HttpResponse::Ok()
                            .cookie(cookie)
                            .json(response);
                    }
                    Err(err) => {
                        return HttpResponse::InternalServerError().json(response);
                    }
                }
            }

            HttpResponse::BadRequest().json(response) // Jika tidak ada user, return 400
        },
        response => HttpResponse::BadRequest().json(response), // Jika gagal login, HTTP 400
    }
}

fn web_scope() -> Scope {
    
    web::scope("/auth")
        .service(login)
}

pub async fn start_server() -> std::io::Result<()> {
    env_logger::init(); // Aktifkan logging
    let secret_key: Key = Key::generate(); 
    dotenvy::dotenv().ok();
    let db_pool = create_pool("db12877").await.expect("Failed to create database pool");
    
    HttpServer::new(move || {
        App::new()
            .service(web::scope("/services")
            .service(web_scope())
        )
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(web::JsonConfig::default().error_handler(generic_services::GenericService::json_error_handler))
        .service(health_check)
        .default_service(route().to(generic_services::GenericService::not_found))
        .wrap(middleware::Logger::default()) // Logging middleware
        .wrap(IdentityMiddleware::default())
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                .cookie_name("token".to_owned())
                .cookie_secure(false)
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(7)))
                .build(),
        )
        .wrap(middleware::NormalizePath::trim()) // 🔥 Normalisasi path (opsional)
        .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8001))?
    .run()
    .await
    .map_err(|e| {
        eprintln!("Server error: {}", e);
        e
    })
}