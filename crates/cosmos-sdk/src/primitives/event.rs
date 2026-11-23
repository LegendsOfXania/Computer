use computer_model::id::EntryId;
use cosmos_sdk_macro::entry;

#[entry(tags("trigger"))]
pub struct TriggerEntry {
    pub triggers: Vec<EntryId>,
}

#[entry(tags("triggerable"))]
pub struct TriggerableEntry {
    #[base]
    pub base: TriggerEntry,
}

#[entry(tags("sequence", "action"))]
pub struct ActionEntry {
    #[base]
    pub base: TriggerableEntry,
}
