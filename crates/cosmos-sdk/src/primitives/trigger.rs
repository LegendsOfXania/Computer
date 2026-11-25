use crate::{Interaction, RegisteredEntry};
use computer_model::id::EntryId;
use cosmos_sdk_macro::{entry, tags};

#[entry]
#[tags("trigger")]
pub struct TriggerEntry {
    pub triggers: Vec<EntryId>,
}

#[tags("triggerable")]
pub trait Triggerable: RegisteredEntry {
    fn execute(self, interaction: Interaction) -> impl Future<Output = Result<(), String>> + Send;
}

#[entry]
#[tags("sequence", "action")]
pub struct ActionEntry {
    #[base]
    pub base: TriggerEntry,
    //todo criteria and modifiers
}