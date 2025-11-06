#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;

    #[actix_rt::test]
    async fn test_receive_log_entries() {
        let client = Client::new();

        // Send multiple log entries with different IPs
        let payloads = [
            json!({"timestamp": "2025-01-01T12:00:00Z", "ip": "192.168.1.1"}),
            json!({"timestamp": "2025-01-01T12:00:01Z", "ip": "192.168.1.2"}),
            json!({"timestamp": "2025-01-01T12:00:02Z", "ip": "192.168.1.1"}),
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
    }
}
