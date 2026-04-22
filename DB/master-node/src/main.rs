mod operations;
mod server;
mod constant;

use std::sync::{Arc, Mutex};
use std::io::{stdout, stdin, Write};
use master_node::lookup::{LookupTable};

use crate::operations::{check_status, kill_all_servers, spawn_servers};
use crate::server::Server;


fn help() {
    println!("Commands");
    println!("  start <N>  -> start N servers");
    println!("  kill       -> kill all servers");
    println!("  status     -> Show status of all servers");
    println!("  help       -> Print all the commands");
}


fn main() {
    let servers = Arc::new(Mutex::new(Vec::<Server>::new()));
    let mut next_port: u16 = 7777;

    let lookupTable = LookupTable::new();

    help();


    loop {
        print!("> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();

        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }


        match parts[0].to_lowercase().as_str() {

            "help" => {
                help();
            }


            "start" => {
                if parts.len() < 2 {
                    println!("Usage: start <number>");
                    continue;
                }

                let count:usize = match parts[1].parse() {
                    Ok(n) => n,
                    Err(_) => {
                        println!("Invalid number");
                        continue;
                    }
                };

                spawn_servers(&servers, count, &mut next_port);
            }

            "kill" => {
                println!("Stopping all servers");
                kill_all_servers(&mut servers.lock().unwrap());
            }

            "status" => {
                println!("Status of all Servers");
                check_status(&servers.lock().unwrap());
            }

            "exit" => {
                println!("Exiting gracefully!");
                kill_all_servers(&mut servers.lock().unwrap());
                break;
            }

            _ => {
                println!("Unknown command. Use 'start <N>' or 'kill'");
            }
        }

    }
}
