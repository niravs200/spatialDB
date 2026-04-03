mod store;

use std::sync::Arc;

use serde_json::Value;
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::store::Store;
use std::io::Result;
use std::str::from_utf8;

#[tokio::main]
async fn main() -> Result<()>{    
    let mut args = std::env::args().skip(1);
    let tcp_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let udp_port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    
    let store = Arc::new(Store::new());


    let udp_store = store.clone();
    tokio::spawn(async move {
        start_udp_server(&udp_store, udp_port).await.unwrap();
    });

    start_tcp_server(&store, tcp_port).await?;

    Ok(())
}

async fn start_udp_server(store: &Arc<Store>, port: u16) -> Result<()> {
    let socket = UdpSocket::bind(format!("127.0.0.1:{}",port)).await?;
    println!("UDP Server Listening on 127.0.0.1:7777");

    let mut buf = vec![0u8; 1024];

    loop {
        let (n, peer) = socket.recv_from(&mut buf).await?;
        let msg = from_utf8(&buf[..n]).unwrap().trim().to_string();

        let response = handle(store, &msg);
        socket.send_to(response.as_bytes(), &peer).await?;
    }
}

async fn start_tcp_server(store: &Arc<Store>, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}",port)).await?;
    println!("TCP server listening on 127.0.0.1:8888");

    loop {
        let (socket, _) = listener.accept().await?;
        
         tokio::spawn({
            let store = store.clone();
            async move {
                handle_tcp_connection(socket, &store).await.unwrap();
            }
        });
    }
}


async fn handle_tcp_connection(socket:TcpStream, store: &Arc<Store>) -> Result<()> {
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
        let response = handle(store, msg);

        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;

    }

    Ok(())
}

fn handle(store: &Arc<Store>, msg: &str) -> String {
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