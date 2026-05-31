mod store;

use std::sync::Arc;

use serde_json::Value;
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::select;
use crate::store::Store;
use std::io::Result;
use std::str::from_utf8;
use tokio::sync::Notify;

#[derive(Clone)]
struct ServerContext {
    store: Arc<Store>,
    shutting_down: Arc<Notify>,
    is_control: bool, 
    port: u16
}

impl ServerContext {
    fn new(
        store: Arc<Store>,
        shutting_down: Arc<Notify>,
        is_control: bool,
        port: u16,
    ) -> Self {
        Self {
            store,
            shutting_down,
            is_control,
            port,
        }
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let tcp_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let udp_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let control_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let shutting_down = Arc::new(Notify::new());
    let store = Arc::new(Store::new());

    let udp_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), false, udp_port);

    let tcp_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), false, tcp_port);

    let control_server_context =
        ServerContext::new(store.clone(), shutting_down.clone(), true, control_port);

    tokio::spawn(start_udp_server(udp_server_context));
    tokio::spawn(start_tcp_server(tcp_server_context));
    tokio::spawn(start_tcp_server(control_server_context));

    shutting_down.notified().await;

    Ok(())
}

async fn start_udp_server(server_context:ServerContext) -> Result<()> {

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

                let response = handle(store.clone(), &msg);
                socket.send_to(response.as_bytes(), &peer).await?;
            }
        }
    }
}

async fn start_tcp_server(server_context:ServerContext) -> Result<()> {
    let ServerContext {
        store, 
        shutting_down, 
        port, 
        is_control
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
            handle(store.clone(), msg)
        };

        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;

    }

    Ok(())
}

fn handle(store: Arc<Store>, msg: &str) -> String {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();
    
    if parts.is_empty() {
        return "ERR empty_command".to_string();
    }

    match parts[0].to_uppercase().as_str() {
        "SET" => {
            let value: Value = serde_json::from_str(parts[2])
                .unwrap_or(Value::String(parts[2].to_string()));
            store.set(parts[1].to_string(), value);
            "OK".to_string()
        }

        "GET" => {
            match store.get(parts[1]) {
                Some(val) => format!("OK {}", val.to_string()),
                None      => "ERR not_found".to_string(),
            }
        }

        "DEL" => {
            store.delete(parts[1]);
            "OK".to_string()
        }
        
        "PING" => "PONG".to_string(),
        
        _ => "ERR unknown_command".to_string(),
    }
}


fn control_plane_handle(msg: &str, store: Arc<Store>, shutting_down: Arc<Notify>) -> String {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();

    if parts.is_empty() {
        return "ERR empty_command".to_string();
    }

    match parts[0].to_uppercase().as_str() {

        "SHUTDOWN" => {
            store.clear();
            shutting_down.notify_waiters();
            "OK shutting_down".to_string()            
        } 

        "PING" => "PONG".to_string(),

        _ => "ERR unknown_command".to_string(),
    }
}