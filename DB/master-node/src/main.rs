mod operations;
mod constant;
mod bound_box;
mod lookup;

use std::{io::{Write, stdin, stdout}, sync::Arc};
use crate::{bound_box::{BoundingBox, Coordinate}, lookup::LookupTable};
use tokio::sync::RwLock;

use crate::operations::{lookup_ports, check_status, kill_all_servers, spawn_servers};

fn help() {
    println!("Commands");
    println!("  start <N> -> start N servers");
    println!("  kill -> kill all servers");
    println!("  status -> Show status of all servers");
    println!("  lookup -> Show connection details based on coordinates");
    println!("  help -> Print all the commands");
}

#[tokio::main]
async fn main() {
    let mut next_port: u16 = 7777;

    let lookup_table = Arc::new(RwLock::new(LookupTable::new()));

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
               if parts.len() < 6 {
                    println!("Usage: start <server-count> <left-coordinate> <top-coordinate> <right-coordinate> <bottom-coordinate>");
                    continue
               }

               let server_count: usize = match parts[1].parse::<usize>() {
                    Ok(v) if v % 4 == 0 => v,
                    Ok(_) => {
                        println!("server-count must be divisible by 4");
                        continue;
                    }
                    Err(_) => {
                        println!("Invalid server-count");
                        continue;
                    }
                };

                let left_coordinate: f64 = match parts[2].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid left coordinate");
                        continue;
                    }
                };

                let top_coordinate: f64 = match parts[3].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid top coordinate");
                        continue;
                    }
                };

                let right_coordinate: f64 = match parts[4].parse() {
                    Ok(v) if v > left_coordinate => v,
                    Ok(_) => {
                        println!("Right coordinate cannot be smaller then left coordinate");
                        continue
                    }
                    Err(_) => {
                        println!("Invalid right coordinate");
                        continue;
                    }
                };

                let bottom_coordinate: f64 = match parts[5].parse() {
                    Ok(v) if v < top_coordinate => v,
                    Ok(_) => {
                        println!("Bottom coordinate cannot be bigger then top coordinate");
                        continue
                    }
                    Err(_) => {
                        println!("Invalid right coordinate");
                        continue;
                    }
                };

                let coordinate_boundary = BoundingBox::new(
                    Coordinate::new(left_coordinate, top_coordinate),
                    Coordinate::new(right_coordinate, bottom_coordinate),
                );

               match spawn_servers(
                    lookup_table.clone(),
                    coordinate_boundary,
                    server_count,
                    &mut next_port
                ).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Failed to spawn servers: {}", e);
                        continue;
                    }
                }
            }

            "kill" => {
                println!("Starting server shutdown process");
                if let Err(e) = kill_all_servers(lookup_table.clone()).await {
                    eprintln!("Shutdown failed: {}", e);
                } else {
                    println!("All servers have been shutdown");
                }
            }

            "status" => {
                println!("Fetching Status...");
                check_status(lookup_table.clone()).await;
            }

            "lookup" => {
                if parts.len() < 3 {
                    println!("Usage: lookup <x> <y>");
                    continue;
                }

                let x: f64 = match parts[1].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid x coordinate");
                        continue;
                    }
                };

                let y: f64 = match parts[2].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid y coordinate");
                        continue;
                    }
                };

                let coordinate = Coordinate { x, y };

                lookup_ports(lookup_table.clone(), coordinate).await;
            }

            "exit" => {
                println!("Starting server shutdown process");
                if let Err(e) = kill_all_servers(lookup_table.clone()).await {
                    eprintln!("Shutdown failed: {}", e);
                } else {
                    println!("All servers have been shutdown");
                    println!("Exiting Program")
                }
            }

            _ => {
                println!("Unknown command. Use 'start <N>' or 'kill'");
            }
        }

    }
}
