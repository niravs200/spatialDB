use std::{collections::HashMap, sync::Arc, time::Duration, vec};
use quinn::Connection;
use rustls::pki_types::CertificateDer;
use serde::Serialize;

use crate::network::quic_connect;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Clone)]
pub struct Neighbors {
    ports: HashMap<Direction, u16>,
    connections: HashMap<Direction, Arc<Connection>>,
    cert: CertificateDer<'static>,
}

impl Neighbors {
    pub fn new(
        north: Option<u16>,
        south: Option<u16>,
        west: Option<u16>,
        east: Option<u16>,
        cert: CertificateDer<'static>,
    ) -> Self {
        let mut ports = HashMap::new();

        ports.extend([
            (Direction::North, north),
            (Direction::South, south),
            (Direction::West, west),
            (Direction::East, east),
        ].into_iter().filter_map(|(dir, port)| {
            port.map(|p| (dir, p))
        }));

        Self { 
            ports,
            connections: HashMap::new(),
            cert
        }
    }

    pub fn set_port(&mut self, dir: Direction, port: u16) {
        self.ports.insert(dir, port);
    }

    pub fn get_port(&self, dir: Direction) -> Option<u16> {
        self.ports.get(&dir).copied()
    }

    pub fn get_ports(&self) -> Vec<(Direction, u16)> {
        self.ports
            .iter()
            .map(|(dir, port)| (*dir, *port))
            .collect()
    }

    pub fn get_connection(&self, dir: Direction) -> Option<Arc<Connection>> {
        self.connections.get(&dir).cloned()
    }

    pub fn set_connection(&mut self, dir: Direction, conn: Connection) {
        self.connections.insert(dir, Arc::new(conn));
    }

    pub async fn connect_all(&mut self) {
        for (dir, port) in &self.ports {
            let mut retries = 10;
            loop {
                match quic_connect(*port, self.cert.clone()).await {
                    Ok(conn) => {
                        self.connections.insert(*dir, Arc::new(conn));
                        println!("Connected to {:?} on port {}", dir, port);
                        break;
                    }
                    Err(_) => {
                        retries -= 1;
                        if retries == 0 {
                            eprintln!("{:?} unreachable, giving up", dir);
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Metadata {
    client_port: u16,
    realtime_port: u16,
    control_port: u16,
    replication_port: u16,
    neighbors: Neighbors
}

#[derive(Serialize)]
pub struct MetadataResponse {
    pub client_port: u16,
    pub realtime_port: u16,
    pub control_port: u16,
    pub quic_port: u16,
    pub neighbors: Vec<(String, u16)>,
}

impl Metadata {
    pub fn new(client_port: u16, realtime_port: u16, control_port: u16, replication_port: u16, neighbors: Neighbors) -> Self {
        Self {
            client_port,
            realtime_port,
            control_port,
            replication_port,
            neighbors,
        }
    }

    pub fn get_metadata(&self) -> MetadataResponse {
        MetadataResponse {
            client_port: self.client_port,
            realtime_port: self.realtime_port,
            control_port: self.control_port,
            quic_port: self.replication_port,
            neighbors: self
                .neighbors
                .get_ports()
                .into_iter()
                .map(|(dir, port)| (format!("{:?}", dir), port))
                .collect(),
        }
    }
}