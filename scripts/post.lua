wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"

counter = 0

function request()
    counter = counter + 1
    -- generate an IP like 192.168.1.X cycling every 255
    local ip = string.format("192.168.1.%d", counter % 255)
    local body = string.format('{"timestamp":"2025-01-01T12:00:00Z","ip":"%s"}', ip)
    wrk.body = body
    return wrk.format(nil, "/logs")
end

