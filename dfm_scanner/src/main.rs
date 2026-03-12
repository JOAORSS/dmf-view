mod aliases;
mod normalizer;
mod parser;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn print_usage() {
    eprintln!("Usage: dfm_scanner [--json] [--aliases [--save]] <file.dfm>");
    eprintln!("  --json     Output parsed DFM as JSON");
    eprintln!("  --aliases  Output component aliases JSON (CsSense DB)");
    eprintln!("  --save     Save aliases to component-aliases.json in the dfm directory");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut json_mode = false;
    let mut aliases_mode = false;
    let mut save_mode = false;
    let mut file_path = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--json" => json_mode = true,
            "--aliases" => aliases_mode = true,
            "--save" => save_mode = true,
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            _ => file_path = Some(arg.clone()),
        }
    }

    let file_path = match file_path {
        Some(p) => p,
        None => {
            eprintln!("Error: no input file specified");
            print_usage();
            process::exit(1);
        }
    };

    if aliases_mode {
        let entries = aliases::build_aliases(Path::new(&file_path));

        let map: HashMap<String, String> = entries
            .iter()
            .map(|e| (e.custom_type.clone(), e.vcl_base.clone()))
            .collect();

        let output = serde_json::json!({
            "generated_at": "unknown",
            "aliases": map
        });

        let json_str = serde_json::to_string_pretty(&output).unwrap_or_else(|e| {
            eprintln!("Error serializing aliases to JSON: {}", e);
            process::exit(1);
        });

        if save_mode {
            let dfm_dir = Path::new(&file_path)
                .parent()
                .unwrap_or_else(|| Path::new("."));
            let out_path = dfm_dir.join("component-aliases.json");
            match fs::write(&out_path, &json_str) {
                Ok(_) => println!("{}", out_path.display()),
                Err(e) => {
                    eprintln!("Error writing aliases file: {}", e);
                    process::exit(1);
                }
            }
        } else {
            println!("{}", json_str);
        }
        return;
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let mut component = match parser::parse_dfm(&content) {
        Some(c) => c,
        None => {
            eprintln!("Error: could not parse DFM file '{}'", file_path);
            process::exit(1);
        }
    };

    if json_mode {
        normalizer::normalize_props(&mut component, 0.0, 0.0);
        match serde_json::to_string_pretty(&component) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error serializing to JSON: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Default: just print component tree summary
        print_tree(&component, 0);
    }
}

fn print_tree(comp: &parser::Component, indent: usize) {
    let prefix = " ".repeat(indent);
    println!("{}{}: {}", prefix, comp.name, comp.comp_type);
    for child in &comp.children {
        print_tree(child, indent + 2);
    }
}
