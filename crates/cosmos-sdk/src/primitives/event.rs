use crate::primitives::trigger::TriggerEntry;
use cosmos_sdk_macro::{entry, tags};

#[entry]
#[tags("event")]
pub struct EventEntry {
    #[base]
    pub base: TriggerEntry,
}

#[entry]
#[tags("cancelable_event")]
pub struct CancelableEventEntry {
    #[base]
    pub base: EventEntry,
    pub cancel: bool,
}