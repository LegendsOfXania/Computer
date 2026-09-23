use std::collections::HashMap;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NavId(usize);

type Handler = Callback<(Key, Modifiers), bool>;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Navigation {
    next_id: Signal<usize>,
    items: Signal<Vec<NavId>>,
    focused: Signal<Option<NavId>>,
    handlers: Signal<HashMap<usize, Handler>>,
}

impl Navigation {
    fn register(&self) -> NavId {
        let mut next_id = self.next_id;
        let id = NavId(next_id());
        next_id.set(id.0 + 1);

        let mut items = self.items;
        items.write().push(id);

        id
    }

    fn unregister(&mut self, id: NavId) {
        let mut items = self.items;
        items.write().retain(|item| *item != id);

        let mut focused = self.focused;
        if focused() == Some(id) {
            focused.set(None);
        }

        self.handlers.write().remove(&id.0);
    }

    fn focus(&self, id: NavId) {
        if self.items.read().contains(&id) {
            let mut focused = self.focused;
            focused.set(Some(id));
        }
    }

    fn is_focused(&self, id: NavId) -> bool {
        (self.focused)() == Some(id)
    }

    pub fn next(&self) {
        self.move_by(1);
    }

    pub fn previous(&self) {
        self.move_by(-1);
    }

    fn move_by(&self, offset: isize) {
        let items = self.items.read();

        if items.is_empty() {
            return;
        }

        let index = (self.focused)()
            .and_then(|focused| items.iter().position(|id| *id == focused))
            .unwrap_or_else(|| if offset >= 0 { 0 } else { items.len() - 1 });

        let next = (index as isize + offset).rem_euclid(items.len() as isize) as usize;

        let mut focused = self.focused;
        focused.set(Some(items[next]));
    }

    pub fn dispatch(&self, key: Key, modifiers: Modifiers) -> bool {
        let Some(focused) = (self.focused)() else {
            return false;
        };

        let handler = self.handlers.read().get(&focused.0).copied();
        let Some(handler) = handler else {
            return false;
        };

        handler.call((key, modifiers))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Nav {
    navigation: Navigation,
    id: Signal<Option<NavId>>,
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

    pub fn on_key(&mut self, handler: Handler) {
        let Some(id) = (self.id)() else {
            return;
        };

        self.navigation
            .handlers
            .write()
            .insert(id.0, handler);
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
    let id = use_signal(|| None::<NavId>);

    use_effect({
        let navigation = navigation;
        let mut id = id;

        move || {
            if id().is_none() {
                let nav_id = navigation.register();

                if initial {
                    navigation.focus(nav_id);
                }

                id.set(Some(nav_id));
            }
        }
    });

    use_drop({
        let mut navigation = navigation;
        let id = id;

        move || {
            if let Some(id) = id() {
                navigation.unregister(id);
            }
        }
    });

    Nav { navigation, id }
}