wrk.headers["Content-Type"] = "application/json"

-- Initialize a global counter
counter = 0

function request()
    counter = counter + 1

    -- Generate a "new" IP for each request
    -- Example format: 192.X.Y.Z where X,Y,Z derived from counter
    local X = (counter / (256*256)) % 256
    local Y = (counter / 256) % 256
    local Z = counter % 256
    local ip = string.format("192.%d.%d.%d", X, Y, Z)

    -- Create JSON body
    local body = string.format('{"timestamp":"2025-01-01T12:00:00Z","ip":"%s"}', ip)

    -- Return full POST request
    return wrk.format("POST", "/logs", {["Content-Type"]="application/json"}, body)
end

