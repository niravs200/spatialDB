use lookup_latency_test::shared::Record;
use serde_json::json;


#[cfg(test)]
mod tests {
    use super::*;

    mod record {
        use super::*;

        #[test]
        fn record_peek() {
            let id = "record_001".to_string();
            let data = json!({
                "health": 50.0
            });
            let record: Record = Record::new(id.clone(), data.clone());
            let peek_data = record.peek();
            assert_eq!(peek_data, data);
        }

        #[test]
        fn record_commit() {
            let id = "record_001".to_string();
            let data = json!({
                "health": 50.0
            });
            let record: Record = Record::new(id.clone(), data.clone());
            let new_data = json!({
                "health": 100.0
            });
            record.commit(new_data.clone());
            let peek_data = record.peek();
            assert_eq!(peek_data, new_data);
        }
    }


    mod shard {

        use lookup_latency_test::shared::Shard;

        use super::*;

        #[test]
        fn shard_add() {
            let id = "record_001".to_string();
            let data = json!({
                "health": 50.0
            });
            let shard = Shard::new();
            shard.add(id.clone(), data.clone());
            let fetched_option = shard.get(&id);
            let fetched_record = fetched_option.unwrap().peek();
            assert_eq!(fetched_record, data)
        }

        #[test]
        fn shard_update() {
            let id = "record_001".to_string();
            let data = json!({
                "health": 50.0
            });
            let shard = Shard::new();
            shard.add(id.clone(), data.clone());
            let fetched_option = shard.get(&id);
            let fetched_record = fetched_option.unwrap().peek();
            assert_eq!(fetched_record, data);

            let new_data = json!({
                "health": 100.0
            });
            shard.update(&id, new_data.clone());
            let new_record = shard.get(&id).unwrap().peek();
            assert_eq!(new_record, new_data)
        }

        #[test]
        fn shard_delete() {
            let id = "record_001".to_string();
            let data = json!({
                "health": 50.0
            });
            let shard = Shard::new();
            shard.add(id.clone(), data.clone());
            let fetched_option = shard.get(&id);
            let fetched_record = fetched_option.unwrap().peek();
            assert_eq!(fetched_record, data);

            shard.delete(&id);
            let new_record = shard.get(&id);
            assert!(new_record.is_none())
        }

        #[test]
        fn get_ids() {
            let id = "record_001".to_string();
            let id2 = "record_002".to_string();
            let data = json!({
                "health": 50.0
            });
            let shard = Shard::new();
            shard.add(id.clone(), data.clone());
            shard.add(id2.clone(), data.clone());

            let mut ids = shard.get_ids();
            ids.sort();

            let mut expected = vec![id, id2];
            expected.sort();

            assert_eq!(ids, expected);
        }

        #[test] 
        fn get_usage() {
            let id = "record_001".to_string();
            let id2 = "record_002".to_string();
            let data = json!({
                "health": 50.0
            });
            let shard = Shard::new();
            shard.add(id.clone(), data.clone());
            shard.add(id2.clone(), data.clone());

            let usage: usize = shard.get_usage();
            assert_eq!(usage, 2)
        }
    }
    
}