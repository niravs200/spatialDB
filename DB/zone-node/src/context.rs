use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;

use crate::store::{Store};
use crate::metadata::{ Direction, Metadata };
use tokio::sync::Notify;
use quinn::Connection;
#[derive(Clone)]
pub struct ServerContext {
    pub store: Arc<Store>,
    pub shutting_down: Arc<Notify>,
    pub port: u16,
}

impl ServerContext {
    pub fn new(
        store: Arc<Store>,
        shutting_down: Arc<Notify>,
        port: u16,
    ) -> Self {
        Self {
            store,
            shutting_down,
            port,
        }
    }
}

#[async_trait]
pub trait Handler: Send + Sync + Clone + 'static {
    async fn handle(&self, msg: &str, store: Arc<Store>, shutting_down: Arc<Notify>) -> String;
}

#[derive(Clone)]
pub struct RealtimeHandler;

#[async_trait]
impl Handler for RealtimeHandler {
    async fn handle(&self, msg: &str, store: Arc<Store>, _shutting_down: Arc<Notify>) -> String {
        crate::handle::db_handle(store, msg)
    }
}

#[derive(Clone)]
pub struct ClientHandler;

#[async_trait]
impl Handler for ClientHandler {
    async fn handle(&self, msg: &str, store: Arc<Store>, _shutting_down: Arc<Notify>) -> String {
        crate::handle::db_handle(store.clone(), msg)
    }
}

#[derive(Clone)]
pub struct NeighborConnections { 
    connections: Arc<RwLock<HashMap<Direction, Connection>>>,
}

impl NeighborConnections {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, direction: Direction, connection: Connection) {
        let mut connections = self.connections.write().unwrap();
        connections.insert(direction, connection);
    }
}

#[derive(Clone)]
pub struct ControlHandler {
    pub metadata: Metadata,
    pub neighbor_connections: NeighborConnections 
}

#[async_trait]
impl Handler for ControlHandler {
    async fn handle(&self, msg: &str, store: Arc<Store>, shutting_down: Arc<Notify>) -> String {
        crate::handle::control_plane_handle(
            msg,
            store.clone(),
            self.metadata.clone(),
            self.neighbor_connections.clone(),    
            shutting_down.clone(),
        ).await
    }
}

#[derive(Clone)]
pub struct ReplicationHandler {
    pub neighbor_connections: NeighborConnections 
}

#[async_trait]
impl Handler for ReplicationHandler {
    async fn handle(&self, msg: &str, store: Arc<Store>, _shutting_down: Arc<Notify>) -> String {
        crate::handle::replication_handle(
            store.clone(),
            self.neighbor_connections.clone(),
            msg,
        )
    }
}