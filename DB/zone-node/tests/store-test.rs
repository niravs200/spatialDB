use zone_node::store::Store;
use serde_json::json;


#[cfg(test)]
mod tests {

    use super::*;
    const KEY1: &str = "key1";

    #[test]
    fn test_set_and_get() {
        let store = Store::new();
        let value1 = json!({
            "name": "Alice",
            "age": 25
        });

        store.set(KEY1.to_string(), value1.clone());

        let result = store.get(KEY1);
        assert_eq!(result, Some(value1));
    }

    #[test]
    fn test_delete() {
        let store: Store = Store::new();
        let value1 = json!({
            "name": "Alice",
            "age": 25
        });
        store.set(KEY1.to_string(),value1);
        store.delete(KEY1);
        let result = store.get(KEY1);
        assert_eq!(result, None);
    }

}