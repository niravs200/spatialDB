use std::sync::Arc;

use crate::store::{Store};
use crate::metadata::{ Metadata };
use tokio::sync::Notify;

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
pub trait Handler: Send + Sync + Clone + 'static {
    fn handle(&self, msg: &str, store: Arc<Store>, shutting_down: Arc<Notify>) -> String;
}

#[derive(Clone)]
pub struct RealtimeHandler;

impl Handler for RealtimeHandler {
    fn handle(&self, msg: &str, store: Arc<Store>, _shutting_down: Arc<Notify>) -> String {
        crate::handle::db_handle(store, msg)
    }
}

#[derive(Clone)]
pub struct ClientHandler;

impl Handler for ClientHandler {
    fn handle(&self, msg: &str, store: Arc<Store>, _shutting_down: Arc<Notify>) -> String {
        crate::handle::db_handle(store.clone(), msg)
    }
}

#[derive(Clone)]
pub struct ControlHandler {
    pub metadata: Metadata,
}

impl Handler for ControlHandler {
    fn handle(&self, msg: &str, store: Arc<Store>, shutting_down: Arc<Notify>) -> String {
        crate::handle::control_plane_handle(
            msg,
            store.clone(),
            shutting_down.clone(),
        )
    }
}

#[derive(Clone)]
pub struct ReplicationHandler;

impl Handler for ReplicationHandler {
    fn handle(&self, msg: &str, store: Arc<Store>, _shutting_down: Arc<Notify>) -> String {
        crate::handle::replication_handle(
            store.clone(),
            msg,
        )
    }
}