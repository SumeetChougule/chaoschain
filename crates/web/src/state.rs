use chaoschain_core::{Block, NetworkEvent, ValidationDecision};
use tokio::sync::broadcast;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub event_tx: broadcast::Sender<NetworkEvent>,
    pub validation_history: Arc<Vec<ValidationDecision>>,
}

impl AppState {
    pub fn new(event_tx: broadcast::Sender<NetworkEvent>) -> Arc<Self> {
        Arc::new(Self { 
            event_tx,
            validation_history: Arc::new(Vec::new()),
        })
    }

    pub fn add_validation(&mut self, validation: ValidationDecision) {
        Arc::make_mut(&mut self.validation_history).push(validation);
    }
} 