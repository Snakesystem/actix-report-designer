use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres, Row, Column};
use std::{env, time::Duration};

async fn connect_db(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query("SELECT 1").fetch_one(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("Connected"),
        Err(_) => HttpResponse::InternalServerError().body("Connection Failed"),
    }
}

#[derive(Deserialize)]
struct RetrieveRequest {
    // start_date: String,
    // end_date: String,
    // param1: i32,
    // param2: i32,
    // param3: i32,
}

async fn retrieve_data(pool: web::Data<PgPool>, body: web::Json<RetrieveRequest>) -> impl Responder {
    // let mut tx = match pool.begin().await {
    //     Ok(tx) => tx,
    //     Err(_) => return HttpResponse::InternalServerError().body("Failed to start transaction"),
    // };

    let call_query = "SELECT * FROM rpt_journal_voucher('2024-01-01 00:00:00', '2024-01-01 00:00:00', 1, 100, 200);";

    // Eksekusi stored procedure dan ambil hasilnya
    let result= sqlx::query(call_query)
        .fetch_all(&**pool)
        .await;

    dbg!(&result);
    match result {
        Ok(rows) => {
            dbg!(&rows);
            let columns: Vec<String> = if let Some(first_row) = rows.first() {
                first_row.columns().iter().map(|col| col.name().to_string()).collect()
            } else {
                vec![] // Jika tidak ada data, tetap kembalikan array kosong
            };

            HttpResponse::Ok().json(serde_json::json!({ "columns": columns, "data": [] }))
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch data"),
    }

}

pub type DbPool = Pool<Postgres>;

pub async fn connection_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(60))
        .idle_timeout(Duration::from_secs(60))
        .connect(&database_url).await
}

pub async fn start_server() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let pool: DbPool = connection_pool().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/connect", web::get().to(connect_db))
            .route("/retrieve", web::post().to(retrieve_data))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
