use serde::{Serialize, Deserialize};
use serde_json::Value;
use dashmap::DashMap;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub data: RwLock<Value>,
}

impl Record {
    pub fn new(data: Value) -> Self {
        Record { 
            data: RwLock::new(data)
        }
    } 

    pub fn commit(&self, new_data: Value) {
        let mut transition = self.data.write().unwrap();
        *transition = new_data;
    }

    pub fn peek(&self) -> Value {
        let view = self.data.read().unwrap();
        view.clone()
    }
}
pub struct Shard {
    registry: DashMap<String, Arc<Record>>
}

impl Shard {
    pub fn new() -> Self {
        Shard { 
            registry: DashMap::new(), 
        }
    }

    pub fn add(&self, id: &str, data: Value) -> bool {
        let record: Arc<Record> = Arc::new(Record::new(data));
        self.registry.insert(id.to_string(), record).is_none()
    }

    pub fn get(&self, id: &str) -> Option<Arc<Record>> {
        self.registry.get(id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn get_ids(&self) -> Vec<String> {
        self.registry.iter().map(|entry| entry.key().clone()).collect()
    }

    pub fn get_usage(&self) -> usize {
        self.registry.len()
    }

    pub fn update(&self, id: &str, new_data: Value) -> bool {
        let record: Option<Arc<Record>> = self.registry.get(id).map(|r| Arc::clone(r.value()));

        if let Some(record) = record {
            let mut exclusive_access = record.data.write().unwrap();
            *exclusive_access = new_data;
            true 
        } else {
            false
        }
    }

    pub fn delete(&self, id: &str) -> bool {
        self.registry.remove(id).is_some()
    }
}