use std::{collections::HashMap};
use rustls::pki_types::CertificateDer;
use serde::Serialize;


#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn inverse(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Clone)]
pub struct NeighborInfo {
    pub port: u16,
    pub cert: CertificateDer<'static>,
}

#[derive(Clone)]
pub struct Neighbors {
    entries: HashMap<Direction, NeighborInfo>,
}

impl Neighbors {
    pub fn new(entries: HashMap<Direction, NeighborInfo>) -> Self {
        Self { entries }
    }

    pub fn get_all_ports(&self) -> HashMap<Direction, u16> {
        self.entries
            .iter()
            .map(|(dir, info)| (*dir, info.port))
            .collect()
    }

    pub fn get_all_neighbors(&self) -> HashMap<Direction, NeighborInfo> {
        self.entries.clone()
    }
}

#[derive(Clone)]
pub struct Metadata {
    client_port: u16,
    realtime_port: u16,
    control_port: u16,
    replication_port: u16,
    neighbors: Neighbors,
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
                .get_all_ports()
                .into_iter()
                .map(|(dir, port)| (format!("{:?}", dir), port))
                .collect(),
        }
    }

    // pub fn get_all_ports(&self) -> HashMap<Direction, u16> {
    //     self.neighbors.get_all_ports()
    // }

    pub fn get_all_neighbors(&self) -> HashMap<Direction, NeighborInfo> {
        self.neighbors.get_all_neighbors()
    }
}