use crate::app::domain::entity::{Ai, Portfolio};
use crate::error::Result;
use crate::ports::outbound::repository::ai::AiRepository;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub struct AiService {
    ai_repository: Arc<AiRepository>,
}

impl AiService {
    pub fn new(ai_repository: Arc<AiRepository>) -> Self {
        Self { ai_repository }
    }

    pub async fn get_ai_response(
        &self,
        user_id: Uuid,
        message: String,
        portfolio: Portfolio,
    ) -> Result<Option<Ai>> {
        let ai_response = self
            .ai_repository
            .get_ai_response(user_id, message, portfolio)
            .await?;

        match ai_response {
            Some(ai) => {
                info!("Successfully retrieved ai response: {}", user_id);
                Ok(Some(ai))
            }
            None => {
                warn!("Ai response not found: {}", user_id);
                Ok(None)
            }
        }
    }
}
