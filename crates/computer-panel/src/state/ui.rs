use computer_model::id::{EntryId, PageId};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Win {
    pub id: EntryId,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub struct UiState {
    pub page: Signal<Option<PageId>>,
    pub windows: Signal<Vec<Win>>,
}

const ORIGIN_Y: i32 = 100;

impl UiState {
    pub fn new() -> Self {
        Self {
            page: Signal::new(None),
            windows: Signal::new(Vec::new()),
        }
    }

    pub fn open(&self, id: &EntryId) {
        let mut windows = self.windows;
        let mut wins = windows.write();

        let win = match wins.iter().position(|w| &w.id == id) {
            Some(i) => wins.remove(i),
            None => {
                let n = i32::try_from(wins.len()).unwrap_or(i32::MAX);
                let offset = n.saturating_mul(30);

                Win {
                    id: id.clone(),
                    x: 40_i32.saturating_add(offset),
                    y: ORIGIN_Y.saturating_add(offset),
                }
            }
        };

        wins.push(win);
    }

    pub fn close(&self, id: &EntryId) {
        let mut windows = self.windows;
        windows.write().retain(|w| &w.id != id);
    }

    pub fn focus(&self, id: &EntryId) {
        let mut windows = self.windows;
        let mut wins = windows.write();

        if let Some(i) = wins.iter().position(|w| &w.id == id) {
            if i + 1 != wins.len() {
                let win = wins.remove(i);
                wins.push(win);
            }
        }
    }

    pub fn move_by(&self, id: &EntryId, dx: i32, dy: i32) {
        let mut windows = self.windows;

        if let Some(win) = windows.write().iter_mut().find(|w| &w.id == id) {
            win.x = win.x.saturating_add(dx);
            win.y = win.y.saturating_add(dy).max(0);
        }
    }

    pub fn active(&self) -> Option<EntryId> {
        self.windows.read().last().map(|w| w.id.clone())
    }
}
