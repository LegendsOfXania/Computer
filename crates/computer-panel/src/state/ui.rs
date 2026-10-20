use computer_model::key::EntryKey;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Win {
    pub key: EntryKey,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub struct UiState {
    pub page: Signal<Option<u64>>,
    pub windows: Signal<Vec<Win>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            page: Signal::new(None),
            windows: Signal::new(Vec::new()),
        }
    }

    pub fn open(&self, key: EntryKey) {
        let mut windows = self.windows;
        let mut wins = windows.write();
        let win = match wins.iter().position(|w| w.key == key) {
            Some(i) => wins.remove(i),
            None => {
                let n = wins.len() as i32;
                Win {
                    key,
                    x: 40 + 30 * n,
                    y: 40 + 30 * n,
                }
            }
        };
        wins.push(win);
    }

    pub fn close(&self, key: EntryKey) {
        let mut windows = self.windows;
        windows.write().retain(|w| w.key != key);
    }

    pub fn active(&self) -> Option<EntryKey> {
        self.windows.read().last().map(|w| w.key)
    }
}
