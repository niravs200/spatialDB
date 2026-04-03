use once_cell::sync::Lazy;
use serde_json::json;
use lookup_latency_test::shared::Record;
use lookup_latency_test::lookup_traditional::TraditionalLookUp;

#[cfg(test)]
mod tests {
    use super::*;

    mod lookup_traditional {

        use std::sync::Arc;

        use super::*;

        static TEST_DATA: Lazy<Vec<(&'static str, serde_json::Value)>> = Lazy::new(|| {
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
            for (i, (record_id, data)) in TEST_DATA.iter().enumerate() {
                let shard_id = &shard_ids[i % shard_ids.len()];
                lookup.insert_record(shard_id, record_id, data.clone());
            }
            lookup
        }

        #[test]
        fn record_peek() {
            let lookup: TraditionalLookUp = init();
            let record: Arc<Record> = lookup
                .get_record(TEST_DATA[0].0)
                .expect("Record should exist");

            assert_eq!(record.peek(), TEST_DATA[0].1);
        }

         #[test]
        fn updated_record() {
            let new_data = json!({
                "health": 90.0
            });
            let lookup: TraditionalLookUp = init();
            let result = lookup.update_record(TEST_DATA[0].0, new_data.clone());
            assert_eq!(result, true);
            let record: Arc<Record> = lookup
                .get_record(TEST_DATA[0].0)
                .expect("Record should exist");
            assert_eq!(record.peek(), new_data);
        }

        #[test]
        fn insert_record() {
            let id: &str = "12fgh";
            let new_data = json!({
                "health": 90.0
            });
            let lookup: TraditionalLookUp = init();
            let shard_ids = lookup.shard_ids();
            let shard_id = shard_ids
                .first()
                .expect("No Shard IDs available");          
            let result = lookup.insert_record(shard_id, id, new_data.clone());
            assert_eq!(result, true);
            let record: Arc<Record> = lookup
                .get_record(id)
                .expect("Record should exist");
            assert_eq!(record.peek(), new_data);
        }

        #[test]
        fn delete_record() {
            let lookup: TraditionalLookUp = init();
            lookup.delete_record(TEST_DATA[0].0);
            assert!(lookup.get_record(TEST_DATA[0].0).is_none())
        }



    }
    
}