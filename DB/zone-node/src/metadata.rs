use std::{collections::HashMap, sync::{Arc, RwLock}};
use rustls::pki_types::CertificateDer;
use serde::Serialize;
use uuid::Uuid;


#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Clone)]
pub struct NeighborInfo {
    pub port: u16,
    pub cert: CertificateDer<'static>,
    pub id: Uuid
}

#[derive(Clone)]
pub struct Neighbors {
    entries: HashMap<Direction, Option<NeighborInfo>>,
}

impl Neighbors {
    pub fn new(entries: HashMap<Direction, Option<NeighborInfo>>) -> Self {
        Self { entries }
    }

    pub fn get_all_ports(&self) -> HashMap<Direction, Option<u16>> {
        self.entries
            .iter()
            .map(|(&dir, info)| (dir, info.as_ref().map(|i| i.port)))
            .collect()
    }

    pub fn get_all_neighbors(&self) -> HashMap<Direction, Option<NeighborInfo>> {
        self.entries.clone()
    }

    pub fn find_direction_by_id(&self, target_uuid: &Uuid) -> Option<Direction> {
        self.entries
            .iter()
            .find(|(_, info)| info.as_ref().is_some_and(|i| i.id == *target_uuid))
            .map(|(&direction, _)| direction)
    }
}

#[derive(Clone)]
pub struct Metadata {
    client_port: u16,
    realtime_port: u16,
    control_port: u16,
    replication_port: u16,
    neighbors:  Arc<RwLock<Neighbors>>,
    pub id: Uuid
}

#[derive(Serialize)]
pub struct MetadataResponse {
    pub id: Uuid,
    pub client_port: u16,
    pub realtime_port: u16,
    pub control_port: u16,
    pub quic_port: u16,
    pub neighbors: Vec<(String, Option<u16>)>,
}

impl Metadata {
    pub fn new(id: Uuid, client_port: u16, realtime_port: u16, control_port: u16, replication_port: u16, neighbors: Arc<RwLock<Neighbors>>) -> Self {
        Self {
            id,
            client_port,
            realtime_port,
            control_port,
            replication_port,
            neighbors,
        }
    }

    pub fn get_metadata(&self) -> MetadataResponse {
        let neighbors = self.neighbors.read().unwrap();

        MetadataResponse {
            id: self.id,
            client_port: self.client_port,
            realtime_port: self.realtime_port,
            control_port: self.control_port,
            quic_port: self.replication_port,
            neighbors: neighbors
                .get_all_ports()
                .into_iter()
                .map(|(dir, port)| (format!("{:?}", dir), port)) // `port` is now `Option<u16>`
                .collect(),
        }
    }   

    pub fn get_all_neighbors(&self) -> HashMap<Direction, Option<NeighborInfo>> {
        let neighbors = self.neighbors.read().unwrap();

        neighbors.get_all_neighbors()
    }

    pub fn find_direction_by_id(&self, target_uuid: &Uuid) -> Option<Direction> {
        self.neighbors.read().unwrap().find_direction_by_id(target_uuid)
    }
}