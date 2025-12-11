use std::{fs, io, sync::{Arc, Mutex}};

use mlua::{Function, Lua};
use serde_json::Value;
use thiserror::Error;

#[derive(Clone)]
pub struct Script {
    path: String,
    lua_state: Arc<Mutex<Lua>>,
    pub metadata: ScriptMetadata
}

unsafe impl Send for Script {}
unsafe impl Sync for Script {}

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Mutex locking error: {0}")]
    LockError(String),
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Script error: {0}")]
    Error(String)
}

pub type ScriptResult<T> = Result<T, ScriptError>;

impl Script {
    pub fn new(path: &str) -> ScriptResult<Script> {
        let script = fs::read_to_string(path)?;
        let mut lua_state = Lua::new();
        let _ = lua_state.load(&script).exec()?;

        let schema = Self::run_func(None, "schema", &mut lua_state)?;
        let Some(table) = schema.as_table() else {
            return Err(ScriptError::Error("No valid table in required function schema".to_string()))
        };
        let schema = parse_table(table);

        let name = schema.get("name").unwrap_or(&Value::from("")).to_string();
        let description  = schema.get("description").unwrap_or(&Value::from("")).to_string();
        let args = schema.get("args").unwrap_or(&Value::from("")).clone();
        let metadata = ScriptMetadata { name, description, script_args: args };

        Ok(Script {
            path: path.to_string(),
            lua_state: Arc::new(Mutex::new(lua_state)),
            metadata
        })
    }

    pub fn get_args(&self) -> ScriptResult<mlua::Value> {
        let Ok(mut lua_state) = self.lua_state.lock() else {
            return Err(ScriptError::LockError("Failed to lock lua state".to_string()))
        };

        Ok(Self::run_func(None, "schema", &mut lua_state)?)
    }

    pub async fn execute(&self, req: String, args: String) -> ScriptResult<mlua::Value>  {
        let Ok(mut lua_state) = self.lua_state.lock() else {
            return Err(ScriptError::LockError("Failed to lock lua state".to_string()))
        };

        // Setup script for lua
        let args_json = serde_json::from_str::<Vec<Value>>(&args)?;

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

        let tbl = lua_state.create_table()?;
        for (k, v) in script_args {
            tbl.set(k, v)?;
        }

        let args = (lua_state.create_string(req)?, tbl);
        Ok(Self::run_func(Some(args), "on_request", &mut lua_state)?)
    }

    fn run_func(args: Option<(mlua::String, mlua::Table)>, function: &str, lua: &mut Lua) -> ScriptResult<mlua::Value> {
        let function: Function = lua.globals().get(function)?;

        if let Some(args) = args {
            return Ok(function.call(args)?);
        } else {
            return Ok(function.call(())?);
        }
    }

}

#[derive(Debug, Clone)]
pub struct ScriptMetadata {
    pub name: String,
    pub description: String,
    pub script_args: Value
}

pub fn parse_table(table: &mlua::Table) -> Value {
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
