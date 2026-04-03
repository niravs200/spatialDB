use std::{process::{Command}, sync::{Arc, Mutex}};

use crate::{constant::ZONE_NODE_PATH, server::Server};


pub fn kill_all_servers(servers: &mut Vec<Server>) {
    for server in servers.iter_mut() {
        if let Err(e) = server.child.kill() {
            eprintln!("Failed to kill server: {}", e)
        } else {
            println!("Server killed successfully")
        }
    }
    servers.clear();
}

pub fn spawn_servers(
    servers: &Arc<Mutex<Vec<Server>>>,
    count: usize,
    next_port: &mut u16
) {
    for _ in 0..count {
        let udp_port = *next_port;
        let tcp_port = *next_port + 1000;
        *next_port += 1;

        let child = Command::new(ZONE_NODE_PATH)
            .arg(tcp_port.to_string())
            .arg(udp_port.to_string())
            .spawn()
            .expect("Failed to start zone-node server");

        let server = Server::new(child, tcp_port, udp_port);

        servers.lock().unwrap().push(server);
        println!("Started server on TCP:{}, UDP:{}", tcp_port, udp_port);
    }
}

pub fn check_status(
    servers: &Vec<Server>
) {
    for server in servers.iter() {
        let healthy = server.check_health();

        if healthy {
            println!("Server running on {}", server.tcp_port);
        } else {
            println!("Server on {} is DOWN", server.tcp_port);
        }
    }    
}