#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;
    use serde_json::Value;

    #[actix_rt::test]
    async fn test_receive_log_entries() {
        let client = Client::new();

        // Send multiple log entries with different IPs
        let payloads = [
            json!({"timestamp": "2025-01-01T12:00:00Z", "ip": "192.168.1.1"}),
            json!({"timestamp": "2025-01-01T12:00:01Z", "ip": "192.168.1.2"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.2"}), //repeated
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.2"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.3"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.4"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.5"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.6"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.7"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.8"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.9"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.10"}),
        ];

        for p in payloads.iter() {
            let resp = client
                .post("http://127.0.0.1:5000/logs")
                .json(p)
                .send()
                .await
                .expect("Request failed");
            assert_eq!(resp.status(), 202);
        }

        let resp = client
            .get("http://127.0.0.1:9102/metrics")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        //assert!(resp.status().is_success());

        //let json: Value = resp.json().await.expect("Invalid JSON");

        //// Ensure the JSON has the expected key
        //assert!(json.get("unique_ip_estimate").is_some());
        //assert!(json["unique_ip_estimate"].as_f64().unwrap() > 9.0);
        //assert!(json["unique_ip_estimate"].as_f64().unwrap() < 11.0);
        // find the gauge line
        let line = resp
            .lines()
            .find(|l| l.starts_with("unique_ip_addresses"))
            .unwrap();
        let value: f64 = line.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(value > 9.0);
        assert!(value < 11.0);
    }
}
