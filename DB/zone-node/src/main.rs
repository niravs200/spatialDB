mod store;
mod handle;
mod network;
mod context;
mod metadata;

use std::sync::Arc;
use std::io::Result;
use tokio::sync::Notify;
use store::Store;
use network::{start_quic_server, start_tcp_server, start_udp_server};
use context::ServerContext;

use crate::{context::{ReplicationContext, ControlPlanContext}, metadata::{Neighbors, Metadata}};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let client_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let realtime_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let control_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let replication_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    
    let north_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let south_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let west_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let east_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);


    let shutting_down = Arc::new(Notify::new());
    let store = Arc::new(Store::new());
    let neighbors =  Neighbors::new(
        Some(north_port),
        Some(south_port),
        Some(west_port),
        Some(east_port),
    );

    let client_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), realtime_port);

    let realtime_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), client_port);

    let replication_server_context = ReplicationContext::new(
        ServerContext::new(
            store.clone(),
            shutting_down.clone(),
            replication_port,
        ),
        neighbors.clone()
    );
    
    let control_server_context = ControlPlanContext::new(
        ServerContext::new(
            store.clone(), 
            shutting_down.clone(), 
            control_port), 
            Metadata::new(client_port, realtime_port, control_port, replication_port, neighbors.clone()));

    tokio::spawn(start_udp_server(client_server_context));
    tokio::spawn(start_tcp_server(realtime_server_context, false));
    tokio::spawn(start_tcp_server(control_server_context, true));
    tokio::spawn(start_quic_server(replication_server_context));

    shutting_down.notified().await;

    Ok(())
}