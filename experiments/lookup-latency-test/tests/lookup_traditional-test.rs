use lookup_latency_test::shared::Record;
use lookup_latency_test::lookup_traditional::TraditionalLookUp;
use once_cell::sync::Lazy;

use serde_json::json;


#[cfg(test)]
mod tests {
    use super::*;

    mod lookup_traditional {

        use std::sync::Arc;

        use super::*;

        static SHARDS_ARRAY: Lazy<Vec<(&'static str, serde_json::Value)>> = Lazy::new(|| {
            vec![
                ("12ewr", json!({"health": 50.0})),
                ("345fs", json!({"health": 60.0})),
                ("31sds", json!({"health": 70.0})),
                ("45ass", json!({"health": 80.0})),
                ("54fas", json!({"health": 90.0})),
            ]
        });

        fn init() -> TraditionalLookUp {
            let lookup: TraditionalLookUp = TraditionalLookUp::new(5);
            let shard_ids: Vec<String> = lookup.shard_ids();
            for (i, (record_id, data)) in SHARDS_ARRAY.iter().enumerate() {
                let shard_id = &shard_ids[i % shard_ids.len()];
                lookup.insert_record(shard_id, record_id, data.clone());
            }
            lookup
        }

        #[test]
        fn record_peek() {
            let lookup: TraditionalLookUp = init();
            let record: Arc<Record> = lookup
                .get_record(SHARDS_ARRAY[0].0)
                .expect("Record should exist");

            assert_eq!(record.peek(), SHARDS_ARRAY[0].1);
        }
    }
    
}