use computer_model::protocol::{
    event::Event,
    request::Request,
};

use crate::data::library;

pub fn handle(
    request: Request,
) -> (Result<(), String>, Option<Event>) {
    match request {
        Request::CreatePage { page } => {
            let result = library::create_page(page.clone())
                .map_err(|error| {
                    tracing::error!("Could not create page: {error}");
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::PageCreated { page });

            (result.map(|_| ()), event)
        }

        Request::UpdatePage { page } => {
            let result = library::edit_page(page.clone())
                .map_err(|error| {
                    tracing::error!("Could not update page: {error}");
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::PageUpdated { page });

            (result.map(|_| ()), event)
        }

        Request::DeletePage { page_id } => {
            let result = library::delete_page(page_id)
                .map_err(|error| {
                    tracing::error!(
                        "Could not delete page {page_id}: {error}"
                    );
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::PageDeleted { page_id });

            (result.map(|_| ()), event)
        }

        Request::CreateEntry { entry } => {
            let result = library::create_entry(entry.clone())
                .map_err(|error| {
                    tracing::error!("Could not create entry: {error}");
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::EntryCreated { entry });

            (result.map(|_| ()), event)
        }

        Request::UpdateEntry { entry } => {
            let result = library::edit_entry(entry.clone())
                .map_err(|error| {
                    tracing::error!("Could not update entry: {error}");
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::EntryUpdated { entry });

            (result.map(|_| ()), event)
        }

        Request::DeleteEntry { key } => {
            let result = library::delete_entry(key)
                .map_err(|error| {
                    tracing::error!(
                        "Could not delete entry {key:?}: {error}"
                    );
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::EntryDeleted { key });

            (result.map(|_| ()), event)
        }

        Request::MoveEntry { key, page_id } => {
            let result = library::move_entry(key, page_id)
                .map_err(|error| {
                    tracing::error!(
                        "Could not move entry {key:?} to page {page_id}: {error}"
                    );
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::EntryMoved { key, page_id });

            (result.map(|_| ()), event)
        }

        Request::Push => {
            let result = library::push()
                .map_err(|error| {
                    tracing::error!("Could not push library: {error}");
                    error.to_string()
                });

            let event = result
                .as_ref()
                .ok()
                .map(|_| Event::Pushed);

            (result.map(|_| ()), event)
        }
    }
}