use std::sync::Arc;
use crate::store::Store; 
use tokio::sync::Notify;

#[derive(Clone)]
pub struct ServerContext {
    pub store: Arc<Store>,
    pub shutting_down: Arc<Notify>,
    pub is_control: bool, 
    pub port: u16
}

impl ServerContext {
    pub fn new(
        store: Arc<Store>,
        shutting_down: Arc<Notify>,
        is_control: bool,
        port: u16,
    ) -> Self {
        Self {
            store,
            shutting_down,
            is_control,
            port,
        }
    }
}
