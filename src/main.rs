use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
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

#[get("/metrics")]
async fn metrics(data: web::Data<Arc<AppState>>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "unique_ip_estimate": data.hll.lock().unwrap().len()
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let hll = HyperLogLog::new(0.001); // 0.1% error rate
    let state = Arc::new(AppState {
        hll: Mutex::new(hll),
    });
    let state_for_logs = state.clone();
    let state_for_metrics = state.clone();

    let logs_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state_for_logs.clone()))
            .service(receive_log)
    })
    .workers(num_cpus::get()) // The point of having 2 servers is to give more resources to log ingestion.
    .bind(("0.0.0.0", 5000))?
    .run();

    // Server for metrics on port 9102
    let metrics_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state_for_metrics.clone()))
            .service(metrics)
    })
    .workers(1)
    .bind(("0.0.0.0", 9102))?
    .run();

    tokio::try_join!(logs_server, metrics_server)?;

    info!("Servers starting: logs on :5000, metrics on :9102");

    Ok(())
}
