
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
    entries: Vec<Arc<RwLock<Entry>>>,
    bounds: Arc<RwLock<Option<BoundingBox>>>,
}

impl LookupTable {

    pub fn new() -> Self {
        Self { entries: Vec::new(), bounds: Arc::new(RwLock::new(None)) }
    }

    pub fn delete_all(&mut self) {
        self.entries.clear();
        self.bounds = Arc::new(RwLock::new(None));
    }

    pub fn set(&mut self, new_entry: &Entry) -> bool {

        for entry in &self.entries {
            if entry.read().unwrap().bounds.intersects(&new_entry.bounds) {
                println!("Boundary: {:?} conflicting with another entry", new_entry.bounds);
                return false
            }
        }

        self.entries.push(Arc::new(RwLock::new(new_entry.clone())));
        
        let mut bounds = self.bounds.write().unwrap();
        *bounds = Some(match &*bounds {
            Some(existing) => BoundingBox {
                min: existing.min.min(&new_entry.bounds.min),
                max: existing.max.max(&new_entry.bounds.max),
            },
            None => new_entry.bounds.clone(),
        });

        return true
    }

    pub fn get(&self, coordinate: Coordinate) -> Option<Entry> {
        
        let bounds = self.bounds.read().unwrap();
        match &*bounds {
            Some(b) => {
                if !b.contains(&coordinate) {
                    println!("Coordinate is out of bound: {:?}", coordinate);
                    return None
                }
            }
            None => {
                println!("No entries has been initiated");
                return  None;
            }
        }

        for entry_lock in &self.entries {
            let entry = entry_lock.read().unwrap();

            if entry.bounds.contains(&coordinate) {
                return Some(entry.clone())
            }
        }

        None
    }
}