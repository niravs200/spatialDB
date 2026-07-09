mod store;
mod handle;
mod network;
mod context;
mod metadata;
mod certificate;
mod config;
mod requests;

use std::{collections::HashMap, sync::Arc};
use std::io::Result;
use std::fs;
use tokio::sync::Notify;
use store::Store;
use network::{start_tcp_server, start_udp_server, start_quic_server};
use context::{ServerContext, ClientHandler, ControlHandler, RealtimeHandler, NeighborConnections};
use crate::metadata::NeighborInfo;
use crate::{certificate::{ReplicationCredentials, parse_cert, parse_key}, context::ReplicationHandler, metadata::{Direction, Metadata, Neighbors}};
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
     let path = std::env::args()
        .nth(1)
        .expect("Usage: zone-node <config.json>");

    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents)
        .expect("Invalid config JSON");


    let shutting_down = Arc::new(Notify::new());
    let store = Arc::new(Store::new());
    let quic_cert = match parse_cert(config.quic_certificate.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Invalid certificate: {e}");
            return Err(e);
        }
    };
    let quic_key = match parse_key(config.quic_private_key.as_bytes()) {
        Ok(c) => c, 
        Err(e) => {
            eprint!("Invalid private key: {e}");
            return Err(e);
        }
    };
    let replication_credentials = Arc::new(ReplicationCredentials {
        cert: quic_cert,
        key: quic_key,
    });

    let neighbors = extract_neighbors_info(&config)?;

    let realtime_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), config.realtime_port);
    let realtime_handler = RealtimeHandler;

    let client_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), config.client_port);
    let client_handler = ClientHandler;

    let control_server_context = 
        ServerContext::new(store.clone(), shutting_down.clone(), config.control_port);
    
    
    let neighbor_connections = NeighborConnections::new();
    let control_handler = ControlHandler { 
        metadata: Metadata::new(
            config.client_port, 
            config.realtime_port, 
            config.control_port, 
            config.replication_port, 
            neighbors.clone()
        ),
        neighbor_connections: neighbor_connections.clone(),
    };

    let replication_server_context = 
        ServerContext::new(store.clone(), shutting_down.clone(), config.replication_port);
    let replication_handler = ReplicationHandler {
        neighbor_connections: neighbor_connections.clone(),
    };

    tokio::spawn(start_udp_server(realtime_server_context, realtime_handler));
    tokio::spawn(start_tcp_server(client_server_context, client_handler));
    tokio::spawn(start_tcp_server(control_server_context, control_handler));
    tokio::spawn(start_quic_server(replication_server_context, replication_handler, replication_credentials.clone(), neighbor_connections));

    shutting_down.notified().await;

    Ok(())
}

fn extract_neighbors_info(config: &Config) -> Result<Neighbors> {
    let mut entries = HashMap::new();

    for (direction, neighbor_config) in [
        (Direction::North, &config.neighbors.north),
        (Direction::South, &config.neighbors.south),
        (Direction::East, &config.neighbors.east),
        (Direction::West, &config.neighbors.west),
    ] {
        let cert = match parse_cert(neighbor_config.certificate.as_bytes()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Invalid certificate for {:?}: {e}", direction);
                return Err(e);
            }
        };

        entries.insert(direction, NeighborInfo {
            port: neighbor_config.port,
            cert
        });
    }

    Ok(Neighbors::new(entries))
}