
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Coordinate {
    x: i64,
    y: i64,
}

impl Coordinate {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

#[derive(Debug, Clone)]
struct BoundingBox {
    pub min: Coordinate,
    pub max: Coordinate,
}

impl BoundingBox {

    pub fn new(min: Coordinate, max: Coordinate) -> Self {
        Self { min, max }
    }

    fn contains(&self, point: &Coordinate) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max.x < other.min.x
            || self.min.x > other.max.x
            || self.max.y < other.min.y
            || self.min.y > other.max.y)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    bounds: BoundingBox,
    udp_port: u16,
    tcp_port: u16,
}

impl Entry {
    pub fn new(bounds: BoundingBox, udp_port: u16, tcp_port:u16) -> Self {
        Self { bounds, udp_port, tcp_port }
    }
}

pub struct LookupTable {
    entries: Vec<Entry>,
    bounds: Option<BoundingBox>,
}

impl LookupTable {

    pub fn new() -> Self {
        Self { entries: Vec::new(), bounds: None }
    }

    pub fn delete_all(&mut self) {
        self.entries.clear();
        self.bounds = None;
    }

    pub fn set(&mut self, entry: &Entry) -> bool {

        if let Some(bounds) = &self.bounds {
            if bounds.intersects(&entry.bounds) {
                println!("Boundary: {:?} conflicting with another entry", entry.bounds);
                return false
            }
        }


        self.entries.push(entry.clone());
        self.bounds = Some(match &self.bounds {
            Some(b) => BoundingBox { 
                min: b.min.min(&entry.bounds.min), 
                max: b.max.max(&entry.bounds.max), 
            },
            None => entry.bounds.clone()
        });

        println!("Overall boundary updated to {:?}", self.bounds);

        return true
    }

    pub fn get(&self, coordinate: Coordinate) -> Option<Entry> {
        
        let bounds= match &self.bounds {
            None => {
                println!("Coordinate: {:?} is out of bounds", coordinate);
                return  None;
            },
            Some(b) => b,
        };

        if !bounds.contains(&coordinate) {
            println!("Coordinate {:?} is out of bounds", coordinate);
            return None;
        }

        for entry in &self.entries {
            if entry.bounds.contains(&coordinate) {
                return Some(entry.clone());
            }
        }

        println!("No entry found for {:?}", coordinate);
        None
    }
}