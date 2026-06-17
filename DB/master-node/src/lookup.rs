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
pub struct AdjacentNodeInfo {
    pub master_port: u16, 
    pub north_port: Option<u16>, 
    pub south_port: Option<u16>, 
    pub east_port:  Option<u16>, 
    pub west_port:  Option<u16>
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
    bounds: Arc<RwLock<BoundingBox>>,
}

impl LookupTable {

    pub fn new() -> Self {
        Self { entries: Vec::new(), bounds: Arc::new(RwLock::new(BoundingBox::zero())) }
    }

    pub fn adjacent_list(&self) -> Vec<AdjacentNodeInfo>{

        let mut adjacent_list: Vec<AdjacentNodeInfo> = Vec::new();

        let bounds = self.bounds.read().unwrap();



        for entry in &self.entries {
            let entry_guard = entry.read().unwrap();
            let mut info = AdjacentNodeInfo {
                master_port: entry_guard.master_port,
                north_port: None,
                south_port: None,
                east_port: None,
                west_port: None,
            };

            if entry_guard.bounds.min.x > bounds.min.x {
                let neighbor = self.get(Coordinate {
                    x: entry_guard.bounds.min.x - 1.0,
                    y: entry_guard.bounds.min.y,
                });

                if let Some(entry) = neighbor {
                    info.north_port = Some(entry.udp_port);
                }
            }

            if entry_guard.bounds.max.x < bounds.max.x {
                let neighbor = self.get(Coordinate {
                    x: entry_guard.bounds.max.x + 1.0,
                    y: entry_guard.bounds.max.y,
                });

                if let Some(entry) = neighbor {
                    info.south_port = Some(entry.udp_port);
                }
            }

            if entry_guard.bounds.min.y > bounds.min.y {
                let neighbor = self.get(Coordinate {
                    x: entry_guard.bounds.min.x,
                    y: entry_guard.bounds.min.y - 1.0,
                });

                if let Some(entry) = neighbor {
                    info.west_port = Some(entry.udp_port);
                }
            }

            if entry_guard.bounds.max.y < bounds.max.y {
                let neighbor = self.get(Coordinate {
                    x: entry_guard.bounds.max.x,
                    y: entry_guard.bounds.max.y + 1.0,
                });

                if let Some(entry) = neighbor {
                    info.east_port = Some(entry.udp_port);
                }
            }
            

            adjacent_list.push(info);
        } 

        adjacent_list
    }

    pub fn delete_all(&mut self) {
        self.entries.clear();
        self.bounds = Arc::new(RwLock::new(BoundingBox::zero()));
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
        if bounds.is_zero() {
            *bounds = new_entry.bounds.clone();
        } else {
            bounds.min.x = bounds.min.x.min(new_entry.bounds.min.x);
            bounds.min.y = bounds.min.y.min(new_entry.bounds.min.y);
            bounds.max.x = bounds.max.x.max(new_entry.bounds.max.x);
            bounds.max.y = bounds.max.y.max(new_entry.bounds.max.y);
        }

        return true
    }

    pub fn get(&self, coordinate: Coordinate) -> Option<EntryView> {
        
        let bounds = self.bounds.read().unwrap();

        if *&bounds.contains(&coordinate) {
            println!("Coordinate is out of bound: {:?}", coordinate);
            return None
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