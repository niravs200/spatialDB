use crate::context::{ReplicationContext, ServerContext};
use crate::handle::{control_plane_handle, handle_request, replication_handle};
use crate::metadata::Neighbors;
use crate::store::Store;
use std::io::Result;
use std::sync::Arc;
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Notify;
use std::str::from_utf8;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use quinn::Connection;


pub async fn start_udp_server(server_context:ServerContext) -> Result<()> {

    let ServerContext {
        store, 
        shutting_down, 
        port, 
        ..
    } = server_context;

    let socket = UdpSocket::bind(format!("127.0.0.1:{}",port)).await?;
    println!("UDP Server Listening on 127.0.0.1:7777");

    let mut buf = vec![0u8; 1024];

    loop {
        select! {
            _ = shutting_down.notified() => {
                return Ok(());
            }

             res = socket.recv_from(&mut buf) => {
                let (n, peer) = res?;

                let msg = from_utf8(&buf[..n])
                    .unwrap()
                    .trim()
                    .to_string();

                let response = handle_request(store.clone(), &msg);
                socket.send_to(response.as_bytes(), &peer).await?;
            }
        }
    }
}

pub async fn start_tcp_server(server_context:ServerContext, is_control: bool) -> Result<()> {
    let ServerContext {
        store, 
        shutting_down, 
        port, 
    } = server_context;

    let listener = TcpListener::bind(format!("127.0.0.1:{}",port)).await?;
    println!("TCP server listening on 127.0.0.1:8888");

    loop {
        select! {
            _ = shutting_down.notified() => {
                return Ok(());
            }

            accept_result = listener.accept() => {
                let (socket, _) = accept_result?;

                let store = store.clone();
                let task_shutdown = shutting_down.clone();
                let is_control = is_control.clone();

                tokio::spawn(async move {
                    let _ = handle_tcp_connection(
                        socket,
                        store,
                        task_shutdown,
                        is_control,
                    )
                    .await;
                });
            }
        }
    }
}


async fn handle_tcp_connection(socket: TcpStream, store: Arc<Store>, shutting_down: Arc<Notify>, is_control: bool) -> Result<()> {

    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let msg = line.trim();
        let response = if is_control {
            control_plane_handle(msg, store.clone(), shutting_down.clone())
        } else {
            handle_request(store.clone(), msg)
        };

        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;

    }

    Ok(())
}

fn configure_quic_server() -> ServerConfig {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])
        .unwrap();

    let cert_der = CertificateDer::from(cert.cert);
    let key_der = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());

    let mut server_config =
        quinn::ServerConfig::with_single_cert(vec![cert_der], key_der)
            .unwrap();

    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(32_u32.into());

    server_config
}

async fn handle_quic_connection(
    connection: Connection,
    store: Arc<Store>,
    neighbors: Neighbors,
    shutting_down: Arc<Notify>,
) {
    println!("New QUIC connection established");

    loop {
        tokio::select! {
            _ = shutting_down.notified() => {
                println!("QUIC connection shutting down");
                return;
            }

            result = connection.accept_uni() => {
                match result {
                    Ok(mut recv_stream) => {
                        let mut buf = vec![0u8; 1024];

                        match recv_stream.read(&mut buf).await {
                            Ok(Some(n)) => {
                                let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();

                                let response = replication_handle(store.clone(), neighbors.clone(), &msg);

                                if let Ok(mut send) = connection.open_uni().await {
                                    let _ = send.write_all(response.as_bytes()).await;
                                }
                            }
                            Ok(None) => {
                                println!("QUIC stream closed");
                                return;
                            }
                            Err(e) => {
                                eprintln!("QUIC read error: {:?}", e);
                                return;
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("QUIC accept error: {:?}", e);
                        return;
                    }
                }
            }
        }
    }
}

pub async fn start_quic_server(replication_context: ReplicationContext) -> Result<()> {
    let ReplicationContext { base, neighbors } = replication_context;
    let ServerContext { store, shutting_down, port, .. } = base;

    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port,
    );

    let server_config = configure_quic_server();

    let endpoint = Endpoint::server(server_config, addr)?;

    println!("QUIC server listening on {}", addr);

    loop {
        tokio::select! {
            _ = shutting_down.notified() => {
                println!("QUIC server shutting down");
                return Ok(());
            }

            Some(connecting) = endpoint.accept() => {
                let store = store.clone();
                let shutdown = shutting_down.clone();
                let neighbors = neighbors.clone(); 

                tokio::spawn(async move {
                    match connecting.await {
                        Ok(connection) => {
                            handle_quic_connection(connection, store, neighbors, shutdown).await;
                        }
                        Err(e) => {
                            eprintln!("QUIC connection failed: {:?}", e);
                        }
                    }
                });
            }
        }
    }
}
