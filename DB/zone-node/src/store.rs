use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde_json::Value;


#[derive(Clone, Default)]
pub struct Store {
    data: Arc<RwLock<HashMap<String, Value>>>,
}

impl Store {
    pub fn new() -> Self { Self::default() }

    pub fn set(&self, key: String, value: Value) {
        self.data.write().unwrap().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.data.read().unwrap().get(key).cloned()
    }
    
    pub fn delete(&self, key: &str) {
        self.data.write().unwrap().remove(key);
    }
}