use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use probabilistic_collections::hyperloglog::HyperLogLog;
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[allow(dead_code)]
#[derive(Deserialize)]
struct LogEntry {
    timestamp: String,
    ip: String,
}

struct AppState {
    shard_hlls: Vec<Arc<Mutex<HyperLogLog<String>>>>,
    senders: Vec<mpsc::Sender<String>>,
    gauge: Gauge,
    registry: Registry,
    num_shards: usize,
}

#[post("/logs")]
async fn receive_log(data: web::Data<Arc<AppState>>, body: web::Json<LogEntry>) -> impl Responder {
    let shard_idx = hash_ip(&body.ip) % data.num_shards;
    let _ = data.senders[shard_idx].send(body.ip.clone()).await;
    HttpResponse::Accepted().finish()
}

#[get("/metrics")]
async fn metrics(data: web::Data<Arc<AppState>>) -> impl Responder {
    let mut merged = HyperLogLog::new(0.001);

    // Merge all shards on-demand
    for shard in &data.shard_hlls {
        let hll = shard.lock().unwrap();
        merged.merge(&*hll);
    }

    // Update Prometheus gauge with merged estimate
    data.gauge.set(merged.len());

    let encoder = TextEncoder::new();
    let metric_families = data.registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}

fn hash_ip(ip: &str) -> usize {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ip.hash(&mut hasher);
    hasher.finish() as usize
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let num_shards = 4;
    let registry = Registry::new();
    let gauge = Gauge::new(
        "unique_ip_addresses",
        "Estimated number of unique IP addresses",
    )
    .unwrap();
    registry.register(Box::new(gauge.clone())).unwrap();

    let mut shard_hlls = Vec::new();
    let mut senders = Vec::new();

    for _ in 0..num_shards {
        let (tx, mut rx) = mpsc::channel::<String>(100_000);
        let hll = Arc::new(Mutex::new(HyperLogLog::new(0.1)));
        shard_hlls.push(hll);
        let shard_ref = shard_hlls.last().unwrap().clone();

        tokio::spawn(async move {
            while let Some(ip) = rx.recv().await {
                let mut hll = shard_ref.lock().unwrap();
                hll.insert(&ip);
            }
        });

        senders.push(tx);
    }

    let state = Arc::new(AppState {
        shard_hlls,
        senders,
        gauge,
        registry,
        num_shards,
    });
    let state_for_logs = state.clone();
    let state_for_metrics = state.clone();

    let logs_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state_for_logs.clone()))
            .service(receive_log)
    })
    .workers(num_cpus::get())
    .bind(("0.0.0.0", 5000))?;

    let metrics_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state_for_metrics.clone()))
            .service(metrics)
    })
    .workers(1)
    .bind(("0.0.0.0", 9102))?;

    info!(
        "Servers starting: logs on :5000, metrics on :9102 with {} shards",
        num_shards
    );

    tokio::try_join!(logs_server.run(), metrics_server.run())?;

    Ok(())
}
