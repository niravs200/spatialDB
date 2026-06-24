mod store;
mod handle;
mod network;
mod context;
mod metadata;
mod certificate;
mod config;

use std::sync::Arc;
use std::io::Result;
use std::fs;
use tokio::sync::Notify;
use store::Store;
use network::{start_quic_server, start_tcp_server, start_udp_server};
use context::{ServerContext, ClientHandler, ControlHandler, RealtimeHandler, ReplicationHandler};
use crate::{certificate::{parse_cert, parse_key}, metadata::{Metadata, Neighbors}};
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
    let neighbors =  Neighbors::new(
        Some(config.neighbors.north_port),
        Some(config.neighbors.south_port),
        Some(config.neighbors.west_port),
        Some(config.neighbors.east_port),
        quic_cert.clone()
    );

    let realtime_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), config.realtime_port);
    let realtime_handler = RealtimeHandler;

    let client_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), config.client_port);
    let client_handler = ClientHandler;

    let control_server_context = 
        ServerContext::new(store.clone(), shutting_down.clone(), config.control_port);
    let control_handler = ControlHandler{ metadata: Metadata::new(config.client_port, config.realtime_port, config.control_port, config.replication_port, neighbors.clone())};
           
    let replication_server_context =
        ServerContext::new(store.clone(),shutting_down.clone(), config.replication_port);
    let replication_handler = ReplicationHandler;



    tokio::spawn(start_udp_server(realtime_server_context, realtime_handler));
    tokio::spawn(start_tcp_server(client_server_context, client_handler));
    tokio::spawn(start_tcp_server(control_server_context, control_handler));
    tokio::spawn(start_quic_server(replication_server_context, replication_handler, quic_cert, quic_key));

    shutting_down.notified().await;

    Ok(())
}