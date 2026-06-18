mod store;
mod handle;
mod network;
mod context;

use std::sync::Arc;
use std::io::Result;
use tokio::sync::Notify;
use store::Store;
use network::{start_quic_server, start_tcp_server, start_udp_server};
use context::ServerContext;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let tcp_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let udp_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let control_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let quic_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);


    let shutting_down = Arc::new(Notify::new());
    let store = Arc::new(Store::new());

    let udp_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), false, udp_port);

    let tcp_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), false, tcp_port);

    let quic_server_context = ServerContext::new(store.clone(), shutting_down.clone(), false, quic_port);

    let control_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), true, control_port);

    tokio::spawn(start_udp_server(udp_server_context));
    tokio::spawn(start_tcp_server(tcp_server_context));
    tokio::spawn(start_tcp_server(control_server_context));
    tokio::spawn(start_quic_server(quic_server_context));

    shutting_down.notified().await;

    Ok(())
}