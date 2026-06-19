use std::collections::HashMap;


#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Clone)]
pub struct Neighbors {
    map: HashMap<Direction, u16>,
}

impl Neighbors {
    pub fn new(
        north: Option<u16>,
        south: Option<u16>,
        west: Option<u16>,
        east: Option<u16>,
    ) -> Self {
        let mut map = HashMap::new();

        map.extend([
            (Direction::North, north),
            (Direction::South, south),
            (Direction::West, west),
            (Direction::East, east),
        ].into_iter().filter_map(|(dir, port)| {
            port.map(|p| (dir, p))
        }));

        Self { map }
    }

    pub fn set(&mut self, dir: Direction, port: u16) {
        self.map.insert(dir, port);
    }

    pub fn get(&self, dir: Direction) -> Option<u16> {
        self.map.get(&dir).copied()
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

    pub fn tcp_port(&self) -> u16 {
        self.client_port
    }

    pub fn udp_port(&self) -> u16 {
        self.realtime_port
    }

    pub fn control_port(&self) -> u16 {
        self.control_port
    }

    pub fn quic_port(&self) -> u16 {
        self.replication_port
    }

    pub fn neighbor_port(&self, dir: Direction) -> Option<u16> {
        self.neighbors.get(dir)
    }
}