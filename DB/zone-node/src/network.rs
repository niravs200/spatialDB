use crate::certificate::pinned_verifier;
use crate::context::{Handler, ServerContext};
use crate::store::Store;
use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Notify;
use std::str::from_utf8;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use quinn::Connection;


pub async fn start_udp_server<H>(
    server_context: ServerContext,
    handler: H,
) -> Result<()>
where
    H: Handler,
{
    let port = server_context.port;

    let socket = UdpSocket::bind(format!("127.0.0.1:{}",port)).await?;
    println!("UDP Server Listening on 127.0.0.1:7777");

    let mut buf = vec![0u8; 1024];

    loop {

        let shutting_down = server_context.shutting_down.clone();
        let store = server_context.store.clone();



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
 
                let response = handler.handle(&msg, store, shutting_down);
                socket.send_to(response.as_bytes(), &peer).await?;
            }
        }
    }
}

pub async fn start_tcp_server<H>(server_context:ServerContext, handler: H) -> Result<()> 
where 
    H: Handler
{
    let port = server_context.port;


    let listener = TcpListener::bind(format!("127.0.0.1:{}",port)).await?;
    println!("TCP server listening on 127.0.0.1:8888");

    loop {

            
        let shutting_down = server_context.shutting_down.clone();
        let store = server_context.store.clone();
        let handler = handler.clone();

        select! {
            _ = shutting_down.notified() => {
                return Ok(());
            }

            accept_result = listener.accept() => {
                let (socket, _) = accept_result?;

                tokio::spawn(async move {
                    let _ = handle_tcp_connection(
                        socket,
                        store,
                        shutting_down,
                        handler,
                    )
                    .await;
                });
            }
        }
    }
}


async fn handle_tcp_connection<H>(socket: TcpStream, store: Arc<Store>, shutting_down: Arc<Notify>, handler: H) -> Result<()> 
where 
    H: Handler    
{
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
        let response = handler.handle(msg, store.clone(), shutting_down.clone());

        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;

    }

    Ok(())
}

pub fn configure_quic_server(cert: CertificateDer<'static>, key: PrivateKeyDer<'static>) -> Result<ServerConfig> {
    let mut server_config = ServerConfig::with_single_cert(vec![cert], key)
        .map_err(|e| Error::new(ErrorKind::Other, format!("quinn server config error: {e}")))?;

    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(32_u32.into());

    Ok(server_config)
}

async fn handle_quic_connection<H>(
    connection: Connection,
    store: Arc<Store>,
    shutting_down: Arc<Notify>,
    handler: H
) where 
    H: Handler
{
    println!("New QUIC connection established");

    loop {
        let store = store.clone();
        let shutting_down = shutting_down.clone();
        let handler = handler.clone();

        tokio::select! {
            _ = shutting_down.notified() => {
                println!("QUIC connection shutting down");
                return;
            }

            result = connection.accept_bi() => {
                match result {
                    Ok((mut send, mut recv)) => {
                        tokio::spawn(async move {
                            let mut buf = vec![0u8; 1024];

                            match recv.read(&mut buf).await {
                                Ok(Some(n)) => {
                                    let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                                    let response = handler.handle(&msg, store, shutting_down);
                                    
                                    if let Err(e) = send.write_all(response.as_bytes()).await {
                                        eprintln!("QUIC write error: {:?}", e);
                                    }
                                    if let Err(e) = send.finish() {
                                        eprintln!("QUIC finish error: {:?}", e);
                                    }
                                }
                                Ok(None) => {
                                    println!("QUIC stream closed by client");
                                }
                                Err(e) => {
                                    eprintln!("QUIC read error: {:?}", e);
                                }
                            }
                        });
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

pub async fn start_quic_server<H>(
    server_context: ServerContext,
    handler: H,
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
) -> Result<()> 
where 
    H: Handler + Clone + Send + 'static,
{
    let port = server_context.port;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    let server_config = configure_quic_server(cert, key)?;
    let endpoint = Endpoint::server(server_config, addr)?;

    println!("QUIC server listening on {}", addr);

    loop {
        let store = server_context.store.clone();
        let shutdown = server_context.shutting_down.clone();
        let handler = handler.clone();

        tokio::select! {
            _ = shutdown.notified() => {
                println!("QUIC server shutting down");
                return Ok(());
            }

            Some(connecting) = endpoint.accept() => {
                tokio::spawn(async move {
                    match connecting.await {
                        Ok(connection) => {
                            handle_quic_connection(connection, store, shutdown, handler).await;
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

pub fn configure_quic_client(cert: CertificateDer<'static>) -> Result<ClientConfig> {
    let rustls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(pinned_verifier(cert))
        .with_no_client_auth();

    let quic_config = quinn::crypto::rustls::QuicClientConfig::try_from(rustls_config)
        .map_err(|e| Error::new(ErrorKind::Other, format!("QuicClientConfig error: {e}")))?;

    Ok(ClientConfig::new(Arc::new(quic_config)))
}

pub async fn quic_connect(
    port: u16,
    cert: CertificateDer<'static>,
) -> Result<Connection> {
    let mut endpoint = Endpoint::client(
        SocketAddr::from((Ipv4Addr::LOCALHOST, 0))
    ).map_err(|e| {
        eprintln!("Endpoint error: {e}");
        Error::new(ErrorKind::Other, "failed to create endpoint")
    })?;

    let client_config = configure_quic_client(cert).map_err(|e| {
        eprintln!("ClientConfig error: {e}");
        Error::new(ErrorKind::Other, "failed to create client config")
    })?;

    endpoint.set_default_client_config(client_config);

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let connection = endpoint
        .connect(addr, "localhost")
        .map_err(|e| {
            eprintln!("Connect error: {e}");
            Error::new(ErrorKind::Other, "connect failed")
        })?
        .await
        .map_err(|e| {
            eprintln!("Handshake error: {e}");
            Error::new(ErrorKind::Other, "handshake failed")
        })?;

    Ok(connection)
}

pub async fn quic_send(conn: &Connection, msg: &str) -> Result<String> {
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| {
            eprintln!("open_bi error: {e}");
            Error::new(ErrorKind::Other, "open_bi failed")
        })?;

    send.write_all(msg.as_bytes())
        .await
        .map_err(|e| {
            eprintln!("write error: {e}");
            Error::new(ErrorKind::Other, "write failed")
        })?;

    send.finish()
        .map_err(|e| {
            eprintln!("finish error: {e}");
            Error::new(ErrorKind::Other, "finish failed")
        })?;

    let mut buf = Vec::new();
    while let Some(chunk) = recv
        .read_chunk(usize::MAX, true)
        .await
        .map_err(|e| {
            eprintln!("read error: {e}");
            Error::new(ErrorKind::Other, "read failed")
        })?
    {
        buf.extend_from_slice(&chunk.bytes);
    }

    String::from_utf8(buf).map_err(|e| {
        eprintln!("utf8 error: {e}");
        Error::new(ErrorKind::Other, "invalid utf8")
    })
}