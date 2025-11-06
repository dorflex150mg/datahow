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

The service is multi-threaded and uses as many CPU workers for log ingestion as it's available.
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

# Benchmark

## Run benchmark

Run benchmarks with:

```bash
cd scripts
./heavy_traffic.sh # make sure your cwd is scripts
```

That benchmark utilizes 8 threads to generate workload for 30 seconds. Expect an output like:


```text
Running 30s test @ http://localhost:5000/logs
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.01ms    1.65ms  22.56ms   77.00%
    Req/Sec     3.01k   228.79     4.17k    71.58%
  720653 requests in 30.07s, 55.67MB read
Requests/sec:  23967.27
Transfer/sec:      1.85MB
```

Note that this setup shares the server's threads with the benchmark tool, since the log servers utilizes all available CPU cores.

## Results

Resuls with log propability of 0.1:
```text
Running 30s test @ http://localhost:5000/logs
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.08ms    1.84ms  27.13ms   78.41%
    Req/Sec     2.97k   249.68     3.59k    74.33%
  711807 requests in 30.06s, 54.99MB read
Requests/sec:  23676.39
Transfer/sec:      1.83MB
```


Results with log propability of 0.02:
```text
Running 30s test @ http://localhost:5000/logs
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.21ms    2.18ms  59.14ms   82.78%
    Req/Sec     2.91k   322.60     7.20k    73.71%
  696143 requests in 30.09s, 53.78MB read
Requests/sec:  23135.16
Transfer/sec:      1.79MB
```

Results with log propability of 0.01:
```text
Running 30s test @ http://localhost:5000/logs
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.40ms    2.36ms  30.09ms   80.72%
    Req/Sec     2.80k   468.32     4.32k    72.07%
  669966 requests in 30.09s, 51.75MB read
Requests/sec:  22267.56
Transfer/sec:      1.72MB
```

The benchmark demonstrate that only marginal gains are achieved by lowering the log probability, hence we opt for a more optimized.

# Considerations

## Memory Usage

The HyperLogLog implementation used here is memory efficient, requiring only a few kilobytes of memory to maintain the unique IP count with a small error margin.

## Sharding

Evaluation has revealed that sharding improves performance up to 10x, signalling that little parallelism got exploited without a sharding strategy. The current implementation uses a simple modulo-based sharding mechanism based on the hash of the IP address.

## Accuracy vs. Performance

The HyperLogLog algorithm provides a good balance between accuracy and performance. With a standard error of around 0.01%, it is suitable for most applications that require unique counts without the overhead of storing all individual IP addresses. Since the utilization of the data described in the problem is pointed out to be non-critical, we prioritize performance and memory efficiency over absolute accuracy.

## Input Validation

We assume that the input log entries are well-formed JSON objects containing valid IP addresses. In a production environment, additional validation and error handling would be necessary to ensure robustness, but since this is not a user-facing service, we keep it simple for demonstration purposes.
