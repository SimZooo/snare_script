use std::fs;

use mlua::{Function, Lua};
use serde_json::Value;

fn run_func(args: (mlua::String, mlua::Table), function: &str, engine: &mut Engine) -> String {
    let function: Function = engine.state.globals().get(function).unwrap();
    let res: String = function.call(args).unwrap();

    res
}

fn run_script(name: &str, engine: &mut Engine, req: String, args: &str) -> String {
    engine.state = Lua::new();
    let path = &format!("scripts/{}.lua", name);
    let script = fs::read_to_string(path).unwrap();
    let _ = engine.state.load(&script).exec().unwrap();

    // Setup script for lua
    let Ok(args_json) = serde_json::from_str::<Vec<Value>>(args) else {
        eprintln!("Malformed arguments");
        return String::new()
    };

    // Setup script args
    let mut script_args = vec![];
    for arg in args_json {
        let Some(args_obj) = arg.as_object() else {
            continue;
        };
        for (k, v) in args_obj.iter() {
            // Check what type of value we have
            match v {
                Value::String(s) => {
                    script_args.push((k.clone(), s.clone()));
                }
                Value::Bool(b) => {
                    script_args.push((k.clone(), b.to_string()));
                }
                Value::Number(n) => {
                    script_args.push((k.clone(), n.to_string()));
                }
                Value::Object(obj) => {
                    // Handle nested objects like {"String": "simzooo"}
                    // You might want to handle this differently based on your needs
                    if let Some(Value::String(s)) = obj.get("String") {
                        script_args.push((k.clone(), s.clone()));
                    }
                }
                _ => {
                    eprintln!("Unsupported value type for key: {}", k);
                }
            }
        }
    }

    let tbl = engine.state.create_table().unwrap();
    for (k, v) in script_args {
        tbl.set(k, v).unwrap();
    }

    let args = (engine.state.create_string(req).unwrap(), tbl);

    run_func(args, "on_request", engine)
}

struct Engine {
    pub state: Lua
}

impl Engine {
    fn new() -> Self {
        let engine = Engine { state: Lua::new() };
        engine
    }
}

fn main() {
    let mut engine = Engine::new();

    let req: String = fs::read_to_string("req.txt").unwrap();

    // Examples of running with args
    {
        // Args specific to script
        let script_args_raw = r#"[{"connection": {"String": "HEISANN"}}]"#;
        let res = run_script("connection", &mut engine, req.clone(), script_args_raw);
        println!("{}", res);
    }
    {
        // Args specific to script
        let script_args_raw = r#"[{"user_agent": {"String": "simzooo"}}]"#;
        let res = run_script("custom_args", &mut engine, req.clone(), script_args_raw);
        println!("{}", res);
    }
}
