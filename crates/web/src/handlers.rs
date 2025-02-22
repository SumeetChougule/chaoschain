use crate::types::*;
use chaoschain_core::{Block, NetworkEvent, ValidationDecision};
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn handle_validation(
    State(state): State<Arc<AppState>>,
    Json(validation): Json<WebValidationResult>,
) -> impl IntoResponse {
    // Broadcast the validation decision
    let _ = state.event_tx.send(NetworkEvent::ValidationResult {
        validator: validation.validator,
        decision: validation.decision,
    });

    Json(validation)
}

// ... existing code ... 