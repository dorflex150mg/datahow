# Unique IP Logging Service - Short README

This Rust microservice receives JSON log entries via HTTP POST at `/logs` on port 5000.
It maintains an in-memory HyperLogLog to track the number of unique IP addresses.
The service prints each received log entry and the current estimated unique IP count.

The estimated number of unique IPs can be get on the `/metrics` endpoint. 

Required JSON fields in each log entry:

```json
{
    "timestamp": "<ISO8601 timestamp>",
    "ip": "<IP address>"
}
```

Example usage:

```bash
curl -X POST http://localhost:5000/logs \
     -H "Content-Type: application/json" \
     -d '{"timestamp":"2025-01-01T12:00:00Z","ip":"192.168.1.1"}'
```

The service is multi-threaded and uses `num_cpus` workers for log ingestion.
HyperLogLog is configured with ~0.1% error to balance accuracy and memory usage.

# Run

Run with:

```bash
cargo run --release
```

Then POST log entries as shown above to test unique IP tracking.


# Test with scripts

Run the following script to send test log entries:

```bash
./make_traffic.sh
```

To generate 255 unique IP requests. To see the current aggregate estimate of unique IPs, run:

```bash
./read_metrics.sh
```

# Test

Run tests with:

```bash
cargo test
```


