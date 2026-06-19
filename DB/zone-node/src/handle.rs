use serde_json::Value;
use std::sync::Arc;
use crate::{metadata::Neighbors, store::Store};
use tokio::sync::Notify;

pub fn handle_request(store: Arc<Store>, msg: &str) -> String {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();
    
    if parts.is_empty() {
        return "ERR empty_command".to_string();
    }

    match parts[0].to_uppercase().as_str() {
        "SET" => {
            let value: Value = serde_json::from_str(parts[2])
                .unwrap_or(Value::String(parts[2].to_string()));
            store.set(parts[1].to_string(), value);
            "OK".to_string()
        }

        "GET" => {
            match store.get(parts[1]) {
                Some(val) => format!("OK {}", val.to_string()),
                None      => "ERR not_found".to_string(),
            }
        }

        "DEL" => {
            store.delete(parts[1]);
            "OK".to_string()
        }
        
        "PING" => "PONG".to_string(),
        
        _ => "ERR unknown_command".to_string(),
    }
}

pub fn replication_handle(store: Arc<Store>, neighbors: Neighbors, msg: &str) -> String {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();
    
    if parts.is_empty() {
        return "ERR empty_command".to_string();
    }

    match  parts[0].to_uppercase().as_str() {
        "COPY" => {
            //TODO proper logic needs to implemented to copy data over to neighbors port
            let value: Value = serde_json::from_str(parts[2])
                .unwrap_or(Value::String(parts[2].to_string()));
            store.set(parts[1].to_string(), value);
            "OK".to_string()
        }

        _ => "ERR unknown_command".to_string(),
    }
}

pub fn control_plane_handle(msg: &str, store: Arc<Store>, shutting_down: Arc<Notify>) -> String {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();

    if parts.is_empty() {
        return "ERR empty_command".to_string();
    }

    match parts[0].to_uppercase().as_str() {

        "SHUTDOWN" => {
            store.clear();
            shutting_down.notify_waiters();
            "OK shutting_down".to_string()            
        } 

        "PING" => "PONG".to_string(),

        _ => "ERR unknown_command".to_string(),
    }
}