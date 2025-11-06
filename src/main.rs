use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use probabilistic_collections::hyperloglog::HyperLogLog;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct LogEntry {
    timestamp: String,
    ip: String,
    // other fields ignored
}

struct AppState {
    hll: Mutex<HyperLogLog<String>>,
}

#[post("/logs")]
async fn receive_log(data: web::Data<Arc<AppState>>, body: web::Json<LogEntry>) -> impl Responder {
    // Insert the IP into HLL and update gauge
    let estimate = {
        let mut hll = data.hll.lock().unwrap();
        hll.insert(&body.ip);
        hll.len()
    };
    println!(
        "Received log entry: timestamp={}, ip={}, hll_estimate={}",
        body.timestamp, body.ip, estimate
    );
    HttpResponse::Accepted().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let hll = HyperLogLog::new(0.001); // 0.1% error rate
    let state = Arc::new(AppState {
        hll: Mutex::new(hll),
    });
    let state_for_logs = state.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state_for_logs.clone()))
            .service(receive_log)
    })
    .workers(num_cpus::get()) // The point of having 2 servers is to give more resources to log ingestion.
    .bind(("0.0.0.0", 5000))?
    .run()
    .await
    .unwrap();

    info!("Servers starting: logs on :5000, metrics on :9102");

    Ok(())
}
