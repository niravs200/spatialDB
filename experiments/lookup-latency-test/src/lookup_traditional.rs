use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;

use crate::shared::{ Record, Shard};

pub struct TraditionalLookUp {
    pub partitions: DashMap<String, Arc<Shard>>,
    pub record_lookup: DashMap<String, String>
}

impl TraditionalLookUp {
    pub fn new(shard_count: usize) -> Self {
        let registry = Self {
            partitions: DashMap::with_capacity(shard_count),
            record_lookup: DashMap::new()
        };

        for i in 0..shard_count {
            let id = format!("shard_{}", i);
            registry.partitions.insert(id, Arc::new(Shard::new()));
        }

        registry
    }

    pub fn insert_record(&self, shard_id: &str, record_id: &str, data: Value) -> bool {
        if let Some(shard) = self.get_shard_by_record_id(record_id) {
            let added_in_shard = shard.add(record_id, data);
            let added_in_lookup = self.record_lookup.insert(record_id.to_string(), shard_id.to_string()).is_none();
            added_in_shard || added_in_lookup
        } else {
            false
        }
    }

    pub fn get_record(&self, record_id: &str) -> Option<Arc<Record>> {
        let shard = self.get_shard_by_record_id(record_id);
        shard.unwrap().get(record_id)
    }

    pub fn update_record(&self, record_id: &str, data: Value) -> bool {
        let shard = self.get_shard_by_record_id(record_id);
        shard.unwrap().update(record_id, data)
    }

    pub fn delete_record(&self, record_id: &str) -> bool {
        if let Some(shard) = self.get_shard_by_record_id(record_id) {
            let deleted_in_shard = shard.delete(record_id);
            let deleted_in_lookup = self.record_lookup.remove(record_id).is_some();
            deleted_in_shard || deleted_in_lookup
        } else {
            false
        }
    }

    pub fn get_shard(&self, shard_id: &str) -> Option<Arc<Shard>> {
        self.partitions.get(shard_id).map(|shard_ptr| {
            Arc::clone(shard_ptr.value())
        })
    }
    
    pub fn get_shard_by_record_id(&self, record_id: &str) -> Option<Arc<Shard>> {
        let shard_id = self.record_lookup.get(record_id);
        let shard = self.get_shard(&shard_id.unwrap());
        shard
    }
    
    pub fn shard_ids(&self) -> Vec<String> {
        self.partitions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}