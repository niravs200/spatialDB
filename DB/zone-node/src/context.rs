use std::sync::Arc;
use crate::store::Store;
use crate::metadata::{Metadata, Neighbors}; 
use tokio::sync::Notify;

#[derive(Clone)]
pub struct ServerContext {
    pub store: Arc<Store>,
    pub shutting_down: Arc<Notify>,
    pub port: u16
}

#[derive(Clone)]
pub struct ControlPlanContext {
    pub base: ServerContext,
    pub metadata: Metadata
}

#[derive(Clone)]
pub struct ReplicationContext {
    pub base: ServerContext, 
    pub neighbors: Neighbors
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

impl ControlPlanContext {
    pub fn new(
        base: ServerContext,
        metadata: Metadata,
    ) -> Self {
        Self { base, metadata }
    }
}

impl ReplicationContext {
    pub fn new(
        base: ServerContext,
        neighbors: Neighbors
    ) -> Self {
        Self { base, neighbors }
    }
}

