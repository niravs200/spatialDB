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

    pub fn get_record(&self, record_id: &str) -> Option<Arc<Record>> {
        let shard_id = self.record_lookup.get(record_id);
        let shard = self.get_shard(&shard_id.unwrap());
        shard.unwrap().get(record_id)
    }

    pub fn update_record(&self, record_id: &str, data: Value) -> bool {
        let shard_id = self.record_lookup.get(record_id);
        let shard = self.get_shard(&shard_id.unwrap());
        shard.unwrap().update(record_id, data)
    }

    pub fn delete_record(&self, record_id: &str) {
        let shard_id = self.record_lookup.get(record_id);
        let shard = self.get_shard(&shard_id.unwrap());
        shard.unwrap().delete(record_id);
    }

    pub fn get_shard(&self, shard_id: &str) -> Option<Arc<Shard>> {
        self.partitions.get(shard_id).map(|shard_ptr| {
            Arc::clone(shard_ptr.value())
        })
    }
    
    pub fn shard_ids(&self) -> Vec<String> {
        self.partitions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}