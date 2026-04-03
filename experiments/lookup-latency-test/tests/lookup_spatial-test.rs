use once_cell::sync::Lazy;
use serde_json::json;
use lookup_latency_test::shared::Record;
use lookup_latency_test::lookup_spatial::SpatialLookup;

#[cfg(test)]
mod tests {
    use super::*;

    mod lookup_spatial {

        use std::sync::Arc;

        use super::*;

        static TEST_DATA: Lazy<Vec<(u64, serde_json::Value)>> = Lazy::new(|| {
            vec![
                (0,  json!({"health": 50.0})),
                (20, json!({"health": 60.0})),
                (40, json!({"health": 70.0})),
                (60, json!({"health": 80.0})),
                (80, json!({"health": 90.0})),
            ]
        });

        fn init() -> SpatialLookup {
            let lookup = SpatialLookup::new(5, 100);
            for (record_id, data) in TEST_DATA.iter() {
                lookup.insert_record(*record_id, data.clone());
            }
            lookup
        }

        fn get_record(lookup: &SpatialLookup, record_id: u64) -> Option<Arc<Record>> {
            lookup
                .get_shard(record_id)
                .and_then(|shard| shard.get(&record_id.to_string()))
        }

        #[test]
        fn record_peek() {
            let lookup = init();
            let record: Arc<Record> = get_record(&lookup, TEST_DATA[0].0)
                .expect("Record should exist");

            assert_eq!(record.peek(), TEST_DATA[0].1);
        }

        #[test]
        fn updated_record() {
            let new_data = json!({
                "health": 99.0
            });
            let lookup = init();
            let result = lookup.update_record(TEST_DATA[0].0, new_data.clone());
            assert_eq!(result, true);
            let record: Arc<Record> = get_record(&lookup, TEST_DATA[0].0)
                .expect("Record should exist");
            assert_eq!(record.peek(), new_data);
        }

        #[test]
        fn insert_record() {
            let record_id: u64 = 5;
            let new_data = json!({
                "health": 100.0
            });
            let lookup = init();
            let result = lookup.insert_record(record_id, new_data.clone());
            assert_eq!(result, true);
            let record: Arc<Record> = get_record(&lookup, record_id)
                .expect("Record should exist");
            assert_eq!(record.peek(), new_data);
        }

        #[test]
        fn delete_record() {
            let lookup = init();
            let result = lookup.delete_record(TEST_DATA[0].0);
            assert_eq!(result, true);
            assert!(get_record(&lookup, TEST_DATA[0].0).is_none());
        }


    }

}
