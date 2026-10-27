use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::id::InteractionId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionContext {
    pub player_id: Uuid,
    pub context_id: InteractionId,
}
