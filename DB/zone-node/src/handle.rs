use serde_json::Value;
use std::sync::Arc;
use crate::{context::{NeighborConnections}, metadata::Metadata, store::Store};
use tokio::sync::Notify;

pub fn db_handle(store: Arc<Store>, msg: &str) -> String {
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

pub fn replication_handle(store: Arc<Store>, msg: &str) -> String {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();
    
    if parts.is_empty() {
        return "ERR empty_command".to_string();
    }

    match  parts[0].to_uppercase().as_str() {
        "Replicate" => {
            let value: Value = serde_json::from_str(parts[2])
                .unwrap_or(Value::String(parts[2].to_string()));
            store.set(parts[1].to_string(), value);
            "OK".to_string()
        }

        _ => "ERR unknown_command".to_string(),
    }
}


// TODO utilize neighbor connection to copy data to the other zone node. Will be useful for testing. 
pub async fn control_plane_handle(msg: &str, store: Arc<Store>, metadata: Metadata, _neighbor_connections: NeighborConnections, shutting_down: Arc<Notify>) -> String {
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

        "Metadata" => {
            serde_json::to_string(&metadata.get_metadata())
                .unwrap_or_else(|_| "ERR serialization_failed".to_string())
        }

   _    => "ERR unknown_command".to_string(),
    }
}