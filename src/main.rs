use std::{fs, path::Path};

use luajit::State;
use serde_json::{Value, json};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
enum ReturnValue {
    Boolean(bool),
    Integer(i32),
    String(String),
}

#[derive(Debug, Clone, Deserialize)]
enum ArgValue {
    Boolean(bool),
    Integer(i32),
    String(String),
    Table(Vec<(String, ArgValue)>)
}

fn push_arg(arg: ArgValue, state: &mut State) {
    match arg {
        ArgValue::String(s) => state.push(s),
        ArgValue::Boolean(b) => state.push(b),
        ArgValue::Integer(i) => state.push(i),
        ArgValue::Table(i) => {
            state.new_table();
            for (k, v) in i {
                push_arg(v, state);
                state.set_field(-2, &k);
            }
        }
    }
}

fn run_func(args: Vec<ArgValue>, function: &str, state: &mut State, n_res: usize) -> Vec<ReturnValue> {
    state.get_global(function);

    for arg in &args {
        push_arg(arg.clone(), state);
    }

    let res = state.pcall(args.len() as i32, n_res as i32, 0);
    if let Err((status, e)) = res {
        eprintln!("Error from script: {e}. Thread status: {:?}", status);
        return vec![];
    }

    let mut results = vec![];
    for _ in 0..n_res {
        // Pops always from top of stack
        let index = -1;
        if state.is_bool(index) {
            results.push(ReturnValue::Boolean(state.to_bool(index).unwrap()));
        } else if state.is_number(index) {
            results.push(ReturnValue::Integer(state.to_int(index).unwrap()));
        } else if state.is_string(index) {
            results.push(ReturnValue::String(state.to_str(index).unwrap().to_string()));
        }
        state.pop(1);
    }

    results
}

fn run_script(name: &str, state: &mut State, req: String, args: &str) {
    // Setup script for lua
    let Ok(args_json) = serde_json::from_str::<Vec<Value>>(args) else {
        eprintln!("Malformed arguments");
        return;
    };

    // Setup script args
    let mut script_args = vec![];
    for arg in args_json {
        let Some(args_obj) = arg.as_object() else {
            continue;
        };
        for (k, v) in args_obj.iter() {
            let value: ArgValue = serde_json::from_value(v.clone()).unwrap();
            script_args.push((k.clone(), value));
        }
    }
    let path = &format!("scripts/{}.lua", name);
    state.do_file(Path::new(path)).unwrap();

    let args = vec![ArgValue::String(req), ArgValue::Table(script_args)];
    let n = args.len();
    let results = run_func(args, "on_request", state, n);
    println!("{:?}", results);
}

fn main() {
    let req: String = fs::read_to_string("req.txt").unwrap();

    let mut state = State::new();
    state.open_libs();

    // Examples of running with args
    {
        // Args specific to script
        let script_args_raw = r#"[{"connection": {"String": "simzooo"}}]"#;
        run_script("connection", &mut state, req.clone(), script_args_raw);
    }
    {
        // Args specific to script
        let script_args_raw = r#"[{"user_agent": {"String": "simzooo"}}]"#;
        run_script("custom_args", &mut state, req.clone(), script_args_raw);
    }
}
