use computer_model::{
    Entry, EntryDefinition, Value,
    id::{EntryId, PageId},
    page::{Page, PageKind},
    protocol::request::Request,
};
use dioxus::{document, signals::ReadableExt};
use dioxus_i18n::tid;

use crate::{
    components::utils::toast::Toasts,
    state::{app::AppState, ui::UiState},
    ws::Client,
};

#[derive(Clone, Copy)]
pub struct Actions {
    app: AppState,
    ui: UiState,
    client: Client,
    toast: Toasts,
}

async fn confirm(message: String) -> bool {
    let mut eval = document::eval("dioxus.send(window.confirm(await dioxus.recv()))");

    eval.send(message).is_ok() && matches!(eval.recv::<bool>().await, Ok(true))
}

fn link_duplicate(entry: &Entry, source: EntryId, duplicate: EntryId) -> Option<Entry> {
    let (source, duplicate) = (Value::from(source), Value::from(duplicate));
    let mut lists = Vec::new();
    let mut pending = vec![entry.root()];

    while let Some(node) = pending.pop() {
        if matches!(node.value(), Some(Value::List(_))) {
            let has = |value: &Value| node.children().any(|item| item.value() == Some(value));

            if has(&source) && !has(&duplicate) {
                lists.push(node.index());
            }
        }

        pending.extend(node.children());
    }

    if lists.is_empty() {
        return None;
    }

    lists.sort_unstable_by(|a, b| b.cmp(a));

    let mut entry = entry.clone();

    for at in lists {
        entry.insert(at, [duplicate.clone()]);
    }

    Some(entry)
}

impl Actions {
    #[must_use]
    pub fn new(app: AppState, ui: UiState, client: Client, toast: Toasts) -> Self {
        Self {
            app,
            ui,
            client,
            toast,
        }
    }

    fn fail(self, title: &'static str, error: String) {
        self.toast.error(tid!(title), error);
    }

    async fn run(self, title: &'static str, request: Request) -> bool {
        match self.client.request(request).await {
            Ok(()) => true,
            Err(error) => {
                self.fail(title, error);
                false
            }
        }
    }

    pub async fn push(self) {
        match self.client.request(Request::Push).await {
            Ok(()) => self.toast.success(tid!("toast-push-succes")),
            Err(error) => self.fail("toast-push-failed", error),
        }
    }

    pub async fn create_page(self, kind: PageKind) -> Option<PageId> {
        let page = match Page::new(tid!("library-new-page-name"), kind) {
            Ok(page) => page,
            Err(error) => {
                self.fail("toast-create-page-failed", error);
                return None;
            }
        };

        let id = page.id;

        self.run("toast-create-page-failed", Request::CreatePage { page })
            .await
            .then_some(id)
    }

    pub async fn edit_page(self, id: PageId, change: impl FnOnce(&mut Page)) {
        let Some(mut page) = self.app.with_page(id, Page::clone) else {
            return;
        };

        change(&mut page);

        self.run("toast-edit-page-failed", Request::UpdatePage { page })
            .await;
    }

    pub async fn delete_page(self, id: PageId) {
        let next = {
            let library = self.app.library.read();
            let mut pages: Vec<_> = library
                .pages
                .values()
                .map(|page| (page.priority, page.id))
                .collect();

            pages.sort_by_key(|(priority, _)| *priority);

            pages
                .iter()
                .position(|(_, page)| *page == id)
                .and_then(|index| {
                    pages.get(index + 1).or_else(|| {
                        index
                            .checked_sub(1)
                            .and_then(|previous| pages.get(previous))
                    })
                })
                .map(|(_, page)| *page)
        };

        let deleted = self
            .run(
                "toast-delete-page-failed",
                Request::DeletePage { page_id: id },
            )
            .await;

        let mut ui = self.ui;

        if deleted && ui.get_opened_page() == Some(id) {
            ui.set_opened_page(next);
        }
    }

    pub async fn create_entry(
        self,
        definition: &EntryDefinition,
        page_id: PageId,
    ) -> Option<EntryId> {
        let entry = match Entry::try_from(definition) {
            Ok(entry) => entry,
            Err(error) => {
                self.fail("toast-create-entry-failed", error);
                return None;
            }
        };

        let id = entry.id();

        self.run(
            "toast-create-entry-failed",
            Request::CreateEntry { entry, page_id },
        )
        .await
        .then_some(id)
    }

    pub async fn edit_entry(self, id: EntryId, change: impl FnOnce(&mut Entry)) -> bool {
        let Some(mut entry) = self.app.with_entry(id, Entry::clone) else {
            return true;
        };

        change(&mut entry);

        self.run("toast-edit-entry-failed", Request::UpdateEntry { entry })
            .await
    }

    pub async fn delete_entry(self, id: EntryId) {
        if !confirm(tid!("inspector-delete-confirm")).await {
            return;
        }

        let deleted = self
            .run(
                "toast-delete-entry-failed",
                Request::DeleteEntry { entry_id: id },
            )
            .await;

        let mut ui = self.ui;

        if deleted && ui.get_opened_entry() == Some(id) {
            ui.set_opened_entry(&self.app, None);
        }
    }

    pub async fn move_entry(self, entry_id: EntryId, page_id: PageId) {
        let moved = self
            .run(
                "toast-edit-entry-failed",
                Request::MoveEntry { entry_id, page_id },
            )
            .await;

        let mut ui = self.ui;

        if moved {
            ui.set_opened_entry(&self.app, Some(entry_id));
        }
    }

    pub async fn duplicate_entry(self, source: Entry, linked: bool) {
        let Some(page_id) = self
            .app
            .library
            .read()
            .page_of(source.id())
            .map(|page| page.id)
        else {
            return;
        };

        let id = match EntryId::generate() {
            Ok(id) => id,
            Err(error) => return self.fail("toast-create-entry-failed", error),
        };

        let source_id = source.id();

        let entry = Entry {
            id: id.to_string(),
            ..source
        };

        if !self
            .run(
                "toast-create-entry-failed",
                Request::CreateEntry { entry, page_id },
            )
            .await
        {
            return;
        }

        if linked {
            let updates: Vec<_> = self
                .app
                .library
                .read()
                .entries
                .values()
                .filter(|entry| entry.id() != id)
                .filter_map(|entry| link_duplicate(entry, source_id, id))
                .map(|entry| (entry.id(), entry.values))
                .collect();

            for (entry_id, values) in updates {
                if !self
                    .edit_entry(entry_id, move |entry| entry.values = values)
                    .await
                {
                    return;
                }
            }
        }

        let mut ui = self.ui;

        ui.set_opened_entry(&self.app, Some(id));
    }
}
