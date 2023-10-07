-- wrk --timeout 10s -t 10 -c 10 -d 10s -s request.lua http://localhost:3000

wrk.method = "GET"

local non2xxResponses = 0

function setup(thread)
    thread:set("threadid", thread.id)
end

function init(args)
    requests = 0
    non2xxResponses = 0
end

function request()
    requests = requests + 1
    path = "/test"
    return wrk.format("GET", path)
end

function response(status, headers, body)
    if status < 200 or status >= 300 then
        non2xxResponses = non2xxResponses + 1
        io.write("Non-2xx Response [Status: " .. status .. "]\n")
        io.write(body .. "\n")
    end
end

function done(summary, latency, requests)
    io.write("Non-2xx Responses: " .. non2xxResponses .. "\n")
end
