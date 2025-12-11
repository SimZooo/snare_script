use std::fs;

use snare_script::Script;

fn main() {
    let req: String = fs::read_to_string("req.txt").unwrap();

    // Examples of running with args
    {
        // Args specific to script
        let mut script = Script::new("scripts/connection.lua");
        let script_args_raw = r#"[{"connection": {"String": "HEISANN"}}]"#;
        println!("Running script: {} which has args: {}!", script.metadata.name, script.metadata.script_args);
        let res = script.run(req.clone(), script_args_raw.to_string());
    }
    {
        // Args specific to script
        let mut script = Script::new("scripts/custom_args.lua");
        let script_args_raw = r#"[{"user_agent": {"String": "simzooo"}}]"#;
        println!("Running script: {} which has args: {}!", script.metadata.name, script.metadata.script_args);
        let res = script.run(req.clone(), script_args_raw.to_string());
    }
}
