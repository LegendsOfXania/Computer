pub mod key;
pub mod reference;
pub mod store;

use model::EntryData;
use std::any::Any;
use std::fmt::Debug;

pub use key::EntryKey;
pub use reference::Ref;
pub use store::EntryStore;

pub trait Entry: Any + Debug + Send + Sync {
    fn id(&self) -> &str;

    fn data(&self) -> &EntryData;

    #[inline]
    fn entry_type(&self) -> &str {
        self.data().entry_type()
    }
}
