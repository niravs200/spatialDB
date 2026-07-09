use serde_json::Value;
use std::sync::Arc;
use crate::{context::NeighborConnections, metadata::{Direction, Metadata}, network::quic_connect, store::Store};
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

pub fn replication_handle(store: Arc<Store>, _neighbor_connections: NeighborConnections, msg: &str) -> String {
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

fn parse_directions(parts: &[&str]) -> Result<Vec<Direction>, String> {
    for part in parts {
        if let Some(value) = part.strip_prefix("DIRECTIONS=") {

            if value.is_empty() {
                return Err("DIRECTIONS cannot be empty".to_string());
            }

            let mut directions = Vec::new();

            for direction in value.split(',') {
                let direction = match direction.to_uppercase().as_str() {
                    "NORTH" => Direction::North,
                    "SOUTH" => Direction::South,
                    "EAST" => Direction::East,
                    "WEST" => Direction::West,
                    _ => {
                        return Err(format!("invalid direction: {}", direction));
                    }
                };

                directions.push(direction);
            }

            return Ok(directions);
        }
    }

    Err("missing DIRECTIONS parameter".to_string())
}

pub async fn control_plane_handle(msg: &str, store: Arc<Store>, metadata: Metadata, neighbor_connections: NeighborConnections, shutting_down: Arc<Notify>) -> String {
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

        "CONNECT" => {
            let directions = match parse_directions(&parts[1..]) {
                Ok(directions) => directions,

                Err(e) => {
                    return format!("ERR {}", e);
                }
            };

            if directions.is_empty() {
                return "ERR missing_directions".to_string();
            }

            let neighbors = metadata.get_all_neighbors();
            for direction in directions {
                let neighbor = neighbors.get(&direction).unwrap();
                let connection = match quic_connect(
                    neighbor.port,
                    neighbor.cert.clone()
                ).await {
                    Ok(connection) => connection,
                    Err(e) => {
                        return format!("ERR connection_failed: {}", e);
                    }
                };
                neighbor_connections.set(direction, connection);
            };

            "OK connection_started".to_string()
        }

   _    => "ERR unknown_command".to_string(),
    }
}