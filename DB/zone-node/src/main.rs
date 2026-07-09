mod store;
mod handle;
mod network;
mod context;
mod metadata;
mod certificate;
mod config;

use std::{sync::{Arc, RwLock}};
use std::io::Result;
use std::fs;
use tokio::sync::Notify;
use store::Store;
use network::{start_tcp_server, start_udp_server, start_quic_server};
use context::{ServerContext, ClientHandler, ControlHandler, RealtimeHandler, NeighborConnections};
use crate::{certificate::{ReplicationCredentials, parse_cert, parse_key}, config::extract_runtime_neighbors, context::{ReplicationHandler}, metadata::Metadata, network::{establish_replication_mesh}};
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

    let initial_neighbors = extract_runtime_neighbors(&config.neighbors)
        .expect("Failed to initialize topology map from configuration certs");

    let neighbors = Arc::new(
        RwLock::new(
            initial_neighbors
        )
    );

    let realtime_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), config.realtime_port);
    let realtime_handler = RealtimeHandler;

    let client_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), config.client_port);
    let client_handler = ClientHandler;

    let control_server_context = 
        ServerContext::new(store.clone(), shutting_down.clone(), config.control_port);
    
    
    let neighbor_connections = NeighborConnections::new();
    let replication_server_context = 
        ServerContext::new(store.clone(), shutting_down.clone(), config.replication_port);
    let metadata = Metadata::new(
            config.id,        
            config.client_port, 
            config.realtime_port, 
            config.control_port, 
            config.replication_port, 
            neighbors
    );
    let control_handler = ControlHandler { 
        metadata: metadata.clone(),
        neighbor_connections: neighbor_connections.clone(),
    };
    let replication_handler = ReplicationHandler;
    let replication_mesh_context = ServerContext::new(store.clone(), shutting_down.clone(), 0);

    tokio::spawn(start_udp_server(realtime_server_context, realtime_handler));
    tokio::spawn(start_tcp_server(client_server_context, client_handler));
    tokio::spawn(start_tcp_server(control_server_context, control_handler));
    tokio::spawn(start_quic_server(replication_server_context, replication_handler.clone(), replication_credentials.clone(), neighbor_connections.clone(), metadata.clone()));
    tokio::spawn(async move {
        if let Err(e) = establish_replication_mesh(
            replication_mesh_context,
            metadata.clone(),
            neighbor_connections.clone(),
            replication_handler,
        ).await {
            eprintln!("Critical failure in replication mesh supervisor: {:?}", e);
        }
    });
    shutting_down.notified().await;

    Ok(())
}