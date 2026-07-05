use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;
use std::sync::Mutex;
use surrealdb::engine::remote::ws::{Ws, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::{Surreal, Error};
use oxide_core::VehicleTelemetry;

struct AppState {
    db: Surreal<surrealdb::engine::remote::ws::Client>,
}

#[derive(Deserialize)]
struct TimeRange {
    start_time: u64,
    end_time: u64,
}

async fn ingest_telemetry(
    data: web::Data<Mutex<AppState>>,
    telemetry: web::Json<Vec<VehicleTelemetry>>,
) -> impl Responder {
    let db = &data.lock().unwrap().db;
    let records = telemetry.into_inner();
    
    // Batch insert all records into the "telemetry" table
    let created: Result<Vec<surrealdb::sql::Value>, Error> = db.insert("telemetry").content(records).await;
    
    match created {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_telemetry(
    data: web::Data<Mutex<AppState>>,
    path: web::Path<String>,
    query: web::Query<TimeRange>,
) -> impl Responder {
    let db = &data.lock().unwrap().db;
    let vin = path.into_inner();
    let sql = "SELECT * FROM telemetry WHERE vin = $vin AND timestamp >= $start AND timestamp <= $end";
    let mut result = db.query(sql)
        .bind(("vin", vin))
        .bind(("start", query.start_time))
        .bind(("end", query.end_time))
        .await
        .unwrap();

    let telemetry: Vec<VehicleTelemetry> = result.take(0).unwrap();
    HttpResponse::Ok().json(telemetry)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
    db.use_ns("oxide").use_db("telemetry").await.unwrap();

    let app_state = web::Data::new(Mutex::new(AppState { db }));

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/v1/telemetry", web::post().to(ingest_telemetry))
            .route("/api/v1/telemetry/{vin}", web::get().to(get_telemetry))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}