use std::fs;

use mlua::{Function, Lua};
use serde_json::Value;

pub struct Script {
    path: String,
    lua_state: Lua,
    pub metadata: ScriptMetadata
}

impl Script {
    pub fn new(path: &str) -> Script {
        let script = fs::read_to_string(path).unwrap();
        let mut lua_state = Lua::new();
        let _ = lua_state.load(&script).exec().unwrap();

        let schema = Self::run_func(None, "schema", &mut lua_state);
        let table = schema.as_table().unwrap();
        let schema = parse_table(table);

        let name = schema.get("name").unwrap_or(&Value::from("")).to_string();
        let description  = schema.get("description").unwrap_or(&Value::from("")).to_string();
        let args = schema.get("args").unwrap_or(&Value::from("")).clone();
        let metadata = ScriptMetadata { name, description, script_args: args };

        Script {
            path: path.to_string(),
            lua_state,
            metadata
        }
    }

    pub fn run(&mut self, req: String, args: String) -> mlua::Value {

        // Setup script for lua
        let Ok(args_json) = serde_json::from_str::<Vec<Value>>(&args) else {
            eprintln!("Malformed arguments");
            return mlua::Value::default()
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

        let tbl = self.lua_state.create_table().unwrap();
        for (k, v) in script_args {
            tbl.set(k, v).unwrap();
        }

        let args = (self.lua_state.create_string(req).unwrap(), tbl);
        Self::run_func(Some(args), "on_request", &mut self.lua_state)
    }

    fn run_func(args: Option<(mlua::String, mlua::Table)>, function: &str, lua: &mut Lua) -> mlua::Value {
        let function: Function = lua.globals().get(function).unwrap();

        if let Some(args) = args {
            return function.call(args).unwrap();
        } else {
            return function.call(()).unwrap();
        }
    }

}

#[derive(Debug)]
pub struct ScriptMetadata {
    pub name: String,
    pub description: String,
    pub script_args: Value
}

fn parse_table(table: &mlua::Table) -> Value {
    let mut map = serde_json::Map::new();
    for pair in table.pairs::<mlua::Value, mlua::Value>() {
        if let Ok((k, v)) = pair {
            let key = match k {
                mlua::Value::String(s) => s.to_string_lossy().to_string(),
                mlua::Value::Integer(i) => i.to_string(),
                mlua::Value::Boolean(b) => b.to_string(),
                mlua::Value::Number(n) => n.to_string(),
                _ => continue
            };

            let val = match v {
                mlua::Value::String(s) => Value::String(s.to_string_lossy().to_string()),
                mlua::Value::Integer(i) => Value::Number(i.into()),
                mlua::Value::Boolean(b) => Value::Bool(b),
                mlua::Value::Number(n) => Value::Number(serde_json::Number::from_f64(n).unwrap_or(0.into())),
                mlua::Value::Table(t) => parse_table(&t),
                _ => continue
            };

            map.insert(key, val);
        }
    }

    serde_json::Value::Object(map)
}
