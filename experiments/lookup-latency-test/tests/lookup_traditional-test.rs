use lookup_latency_test::shared::Record;
use serde_json::json;


#[cfg(test)]
mod tests {
    use super::*;

    mod lookup_traditional {
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
    
}