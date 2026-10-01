use std::collections::HashMap;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Navigation {
    next_id: Signal<usize>,
    items: Signal<Vec<usize>>,
    focused: Signal<Option<usize>>,
    handlers: Signal<HashMap<usize, Callback<(Key, Modifiers), bool>>>,
}

impl Navigation {
    fn register(&self) -> usize {
        let mut next_id = self.next_id;
        let mut items = self.items;

        let id = next_id();
        next_id.set(id + 1);

        items.write().push(id);

        id
    }

    fn unregister(&self, id: usize) {
        let mut items = self.items;
        let mut handlers = self.handlers;

        items.write().retain(|item| *item != id);
        handlers.write().remove(&id);

        let mut focused = self.focused;

        if focused() == Some(id) {
            focused.set(None);
        }
    }

    fn focus(&self, id: usize) {
        if self.items.read().contains(&id) {
            let mut focused = self.focused;
            focused.set(Some(id));
        }
    }

    fn is_focused(&self, id: usize) -> bool {
        (self.focused)() == Some(id)
    }

    fn move_by(&self, offset: isize) {
        let items = self.items.read();

        if items.is_empty() {
            return;
        }

        let index = (self.focused)()
            .and_then(|focused| items.iter().position(|id| *id == focused))
            .unwrap_or(if offset >= 0 {
                0
            } else {
                items.len() - 1
            });

        let next = (index as isize + offset)
            .rem_euclid(items.len() as isize) as usize;

        let mut focused = self.focused;
        focused.set(Some(items[next]));
    }

    pub fn next(&self) {
        self.move_by(1);
    }

    pub fn previous(&self) {
        self.move_by(-1);
    }

    pub fn dispatch(&self, key: Key, modifiers: Modifiers) -> bool {
        let Some(focused) = (self.focused)() else {
            return false;
        };

        let Some(handler) = self.handlers.read().get(&focused).copied() else {
            return false;
        };

        handler.call((key, modifiers))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Nav {
    navigation: Navigation,
    id: Signal<Option<usize>>,
}

impl Nav {
    pub fn focus(&self) {
        let Some(id) = (self.id)() else {
            return;
        };

        self.navigation.focus(id);
    }

    pub fn focused(&self) -> bool {
        let Some(id) = (self.id)() else {
            return false;
        };

        self.navigation.is_focused(id)
    }

    pub fn on_key(&mut self, handler: Callback<(Key, Modifiers), bool>) {
        let Some(id) = (self.id)() else {
            return;
        };

        let mut handlers = self.navigation.handlers;
        handlers.write().insert(id, handler);
    }
}

pub fn use_nav_root() {
    let navigation = Navigation {
        next_id: use_signal(|| 0),
        items: use_signal(Vec::new),
        focused: use_signal(|| None),
        handlers: use_signal(HashMap::new),
    };

    use_context_provider(|| navigation);
}

pub fn use_nav() -> Nav {
    use_nav_with_focus(false)
}

pub fn use_nav_focus() -> Nav {
    use_nav_with_focus(true)
}

fn use_nav_with_focus(initial: bool) -> Nav {
    let navigation = use_context::<Navigation>();
    let id = use_signal(|| None::<usize>);

    use_effect({
        let mut id = id;
        let navigation = navigation;

        move || {
            if id().is_some() {
                return;
            }

            let nav_id = navigation.register();

            if initial {
                navigation.focus(nav_id);
            }

            id.set(Some(nav_id));
        }
    });

    use_drop({
        let navigation = navigation;
        let id = id;

        move || {
            if let Some(id) = id() {
                navigation.unregister(id);
            }
        }
    });

    Nav { navigation, id }
}