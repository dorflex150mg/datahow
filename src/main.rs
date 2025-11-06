use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use probabilistic_collections::hyperloglog::HyperLogLog;
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
#[derive(Deserialize)]
struct LogEntry {
    timestamp: String,
    ip: String,
}

struct AppState {
    hll: Mutex<HyperLogLog<String>>,
    gauge: Gauge,
    registry: Registry,
}

#[post("/logs")]
async fn receive_log(data: web::Data<Arc<AppState>>, body: web::Json<LogEntry>) -> impl Responder {
    {
        let mut hll = data.hll.lock().unwrap();
        hll.insert(&body.ip);
        let estimate = hll.len();
        data.gauge.set(estimate);
        println!(
            "Received log from IP: {}, estimated unique IPs: {}",
            body.ip, estimate
        );
    }
    HttpResponse::Accepted().finish()
}

#[get("/metrics")]
async fn metrics(data: web::Data<Arc<AppState>>) -> impl Responder {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = data.registry.gather();
    println!("Gathered {:?} metric families", metric_families);
    encoder.encode(&metric_families, &mut buffer).unwrap();
    println!("Metrics: {}", String::from_utf8_lossy(&buffer));
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let registry = Registry::new();
    let gauge = Gauge::new(
        "unique_ip_addresses",
        "Estimated number of unique IP addresses since start",
    )
    .unwrap();
    registry.register(Box::new(gauge.clone())).unwrap();
    let hll = HyperLogLog::new(0.001); // 0.1% error rate
    let state = Arc::new(AppState {
        hll: Mutex::new(hll),
        gauge,
        registry,
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
