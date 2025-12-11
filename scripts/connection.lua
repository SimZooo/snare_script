function schema()
    return {
        name = "Connection Test",
        description = "Test of changing connection header",
        args = {
            connection = "String"
        }
    }
end

function on_request(req, args)
    local new_req = "";
    for i in string.gmatch(req, "[^\r\n]+") do
        local part = string.lower(i)
        if string.match(part, "^%s*connection%s*:%s*keep%-alive") then
            part = "connection: "..args.connection
        end
        new_req = new_req..part.."\r\n"
    end

    return new_req, true
end

function on_response(res)
    return res
end