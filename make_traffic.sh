#!/bin/bash
for i in {1..255}; do
  curl -s -X POST http://localhost:5000/logs \
    -H "Content-Type: application/json" \
    -d "{\"timestamp\":\"2025-01-01T12:00:00Z\",\"ip\":\"192.168.1.$i\",\"url\":\"/\"}" \
  >/dev/null
done

