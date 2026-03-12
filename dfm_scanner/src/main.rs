mod normalizer;
mod parser;

use std::env;
use std::fs;
use std::process;

fn print_usage() {
    eprintln!("Usage: dfm_scanner [--json] <file.dfm>");
    eprintln!("  --json  Output parsed DFM as JSON");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut json_mode = false;
    let mut file_path = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--json" => json_mode = true,
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
