function schema()
    return {
        name = "Custom Args",
        description = "Test of changing user-agent header",
        args = {
            user_agent = "String"
        }
    }
end

function on_request(req, args)
    local new_req = "";
    for i in string.gmatch(req, "[^\r\n]+") do
        local part = string.lower(i)
        if string.match(part, "^%s*user%-agent") then
            part = "user-agent: "..args.user_agent
        end
        new_req = new_req..part.."\r\n"
    end

    return new_req, true
end

function on_response(res)
    return res
end