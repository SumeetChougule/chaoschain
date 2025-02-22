use crate::{handlers::*, types::*};
use chaoschain_core::{Block, NetworkEvent, ValidationDecision};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/validation", post(handle_validation))
        .with_state(state)
}

// ... existing code ... 