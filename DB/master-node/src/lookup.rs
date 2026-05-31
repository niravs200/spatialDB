use std::sync::{Arc, RwLock};

use crate::bound_box::{BoundingBox, Coordinate};

#[derive(Debug, Clone)]
pub struct Entry {
    bounds: BoundingBox,
    udp_port: u16,
    tcp_port: u16,
    master_port: u16
}

impl Entry {
    pub fn new(bounds: BoundingBox, udp_port: u16, tcp_port:u16, master_port: u16) -> Self {
        Self { bounds, udp_port, tcp_port, master_port }
    }
}

#[derive(Debug, Clone)]
pub struct EntryView {
    pub udp_port: u16,
    pub tcp_port: u16,
}

impl EntryView {
    pub fn new(entry: &Entry) -> Self {
        Self { udp_port: entry.udp_port, tcp_port: entry.tcp_port}
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

    pub fn get(&self, coordinate: Coordinate) -> Option<EntryView> {
        
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
                return Some(EntryView::new(&entry));
            }
        }

        None
    }

    pub fn get_all_master_port(&self) -> Vec<u16> {
        let mut master_ports = Vec::new();

        for entry_lock in &self.entries {
        let entry = entry_lock.read().unwrap();
            master_ports.push(entry.master_port);
        }

        master_ports
    }
}