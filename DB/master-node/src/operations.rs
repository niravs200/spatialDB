use std::{io::{Error, ErrorKind}, process::Command, sync::Arc};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpStream, sync::RwLock};
use futures::future::join_all;

use crate::{bound_box::{BoundingBox, Coordinate}, constant::{HEALTH_COMMAND, HEALTH_POSITIVE_CONFIRMATION_RESPONSE, SHUTDOWN_COMMAND, ZONE_NODE_PATH}, lookup::{Entry, LookupTable}};

pub async fn trigger_shutdown(port: u16) -> Result<(), Error> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;

    stream.write_all(SHUTDOWN_COMMAND.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

pub async fn fetch_status(port: u16) -> Result<String, Error> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;

    // send request
    stream.write_all(HEALTH_COMMAND.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    // read response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();

    reader.read_line(&mut response).await?;

    Ok(response.trim().to_string())
}


pub async fn kill_all_servers(lookup_table: Arc<RwLock<LookupTable>>) -> Result<() , Error> {
    let master_ports = lookup_table.read().await.get_all_master_port();

     let tasks = master_ports.into_iter().map(|port| {
        trigger_shutdown(port)
    });

    join_all(tasks).await;

    lookup_table.write().await.delete_all();

    Ok(())
}

pub async fn spawn_servers(
    lookup_table: Arc<RwLock<LookupTable>>,
    area_bound_coordinates: BoundingBox,
    count: usize,
    next_port: &mut u16
) ->  Result<(), std::io::Error> {

    if count % 4 == 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "invalid count"));
    }

    let boxes: Vec<BoundingBox> = area_bound_coordinates.divide_bound_box(count as f64);

    for b in boxes {
        let udp_port = *next_port;
        let tcp_port = *next_port + 1000;
        let master_port = *next_port + 2000;
        *next_port += 1;

        Command::new(ZONE_NODE_PATH)
            .arg(tcp_port.to_string())
            .arg(udp_port.to_string())
            .arg(master_port.to_string())
            .spawn()
            .expect("Failed to start zone-node server");

        println!("Started server on TCP:{}, UDP:{}, Master: {}", tcp_port, udp_port, master_port);

        let entry = Entry::new(b, udp_port, tcp_port, master_port); 
        lookup_table.write().await.set(&entry);
    } 

    Ok(())
}

pub async fn check_status(
    lookup_table: Arc<RwLock<LookupTable>>
) {
    let master_ports = lookup_table.read().await.get_all_master_port();

      for port in master_ports {
        match fetch_status(port).await {
            Ok(response) if response == HEALTH_POSITIVE_CONFIRMATION_RESPONSE => {
                println!("Port {} -> ALIVE", port);
            }

            Ok(response) => {
                println!("Port {} -> UNEXPECTED RESPONSE: {}", port, response);
            }

            Err(e) => {
                println!("Port {} -> DEAD ({})", port, e);
            }
        }
    }
}

pub async fn lookup_ports(
    lookup_table: Arc<RwLock<LookupTable>>,
    coordinate: Coordinate
) {
    let entry_view = lookup_table.read().await.get(coordinate).unwrap();
    println!("UDP: {},  TCP: {}", entry_view.tcp_port, entry_view.udp_port)
}