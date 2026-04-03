use serde_json::Value;

use crate::shared::Shard;

pub struct SpatialLookup {
    total_size: u64,
    shard_size: u64,
    shards: Vec<Shard>
}

impl SpatialLookup {

    pub fn new(shard_count: u64, total_size: u64 ) -> SpatialLookup {
        assert!(shard_count > 0);
        assert!(total_size > 0);
         
        let shard_size = total_size / shard_count;

        let mut shards = Vec::with_capacity(shard_count as usize);
        for _ in 0..shard_count {
            shards.push(Shard::new())
        }

        Self {
            total_size, 
            shard_size,
            shards
        }
    }

    pub fn get_shard(&self, record_id: u64) -> Option<&Shard> {
        if record_id >= self.total_size {
            return None;
        }

        let index = (record_id / self.shard_size) as usize;
        self.shards.get(index)
    }

    pub fn insert_record(&self, record_id: u64, data: Value) -> bool {
        if let Some(shard) = self.get_shard(record_id) {
            let key = record_id.to_string();
            let added_in_shard = shard.add(&key, data);
            added_in_shard
        } else {
            false
        }
    }

    pub fn update_record(&self, record_id: u64 , data: Value) -> bool {
        if let Some(shard) = self.get_shard(record_id) {
            let key = record_id.to_string();
            let updated_in_shard = shard.update(&key,data);
            updated_in_shard
        } else {
            false
        }
    }

    pub fn delete_record(&self, record_id: u64) -> bool {
        if let Some(shard) = self.get_shard(record_id) {
            let key = record_id.to_string();
            let deleted_in_shard = shard.delete(&key);
            deleted_in_shard
        } else {
            false
        }
    }
}