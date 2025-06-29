mod orset;
mod rga;
mod sequential_edit;
mod common;
mod concurrent_edit;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <experiment>", args[0]);
        eprintln!("Available experiments: orset_add1, orset_remove1, rga_insert1, rga_delete1, sequential_edit <dataset>, concurrent_edit <dataset>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "orset_add1" => {
            println!("Running ORSet Add experiment...");
            orset::orset_add1();
        }
        "orset_remove1" => {
            println!("Running ORSet Remove experiment...");
            orset::orset_remove1();
        }
        "rga_insert1" => {
            println!("Running RGA Insert experiment...");
            rga::rga_insert1();
        }
        "rga_delete1" => {
            println!("Running RGA Delete experiment...");
            rga::rga_delete1();
        }
        "sequential_edit" => {
            if args.len() < 3 {
                eprintln!("Usage: {} sequential_edit <dataset>", args[0]);
                eprintln!("Available datasets: friendsforever_flat, json-crdt-patch, sveltecomponent");
                std::process::exit(1);
            }
            sequential_edit::sequential_editing(args[2].as_str());
        }
        "concurrent_edit" => {
            if args.len() < 3 {
                eprintln!("Usage: {} concurrent_edit <dataset>", args[0]);
                eprintln!("Available datasets: friendsforever");
                std::process::exit(1);
            }
            concurrent_edit::run_peers(args[2].as_str());
        }
        _ => {
            eprintln!("Unknown experiment: {}", args[1]);
            eprintln!("Available experiments: orset_add1, orset_remove1, rga_insert1, rga_delete1, sequential_edit <dataset>, concurrent_edit <dataset>");
            std::process::exit(1);
        }
    }
}
