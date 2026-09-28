use computer_model::protocol::{
    event::Event,
    request::Request,
};

use crate::data::library;

pub fn handle(
    _id: u64,
    request: Request,
) -> Result<Option<Event>, ()> {
    match request {
        Request::CreatePage { page } => {
            library::create_page(page.clone())
                .map_err(|error| {
                    tracing::error!(
                        "Could not create page: {error}"
                    );
                })?;

            Ok(Some(Event::PageCreated { page }))
        }

        Request::UpdatePage { page } => {
            library::edit_page(page.clone())
                .map_err(|error| {
                    tracing::error!(
                        "Could not update page: {error}"
                    );
                })?;

            Ok(Some(Event::PageUpdated { page }))
        }

        Request::DeletePage { page_id } => {
            library::delete_page(page_id)
                .map_err(|error| {
                    tracing::error!(
                        "Could not delete page {page_id}: {error}"
                    );
                })?;

            Ok(Some(Event::PageDeleted { page_id }))
        }

        Request::CreateEntry { entry } => {
            library::create_entry(entry.clone())
                .map_err(|error| {
                    tracing::error!(
                        "Could not create entry: {error}"
                    );
                })?;

            Ok(Some(Event::EntryCreated { entry }))
        }

        Request::UpdateEntry { entry } => {
            library::edit_entry(entry.clone())
                .map_err(|error| {
                    tracing::error!(
                        "Could not update entry: {error}"
                    );
                })?;

            Ok(Some(Event::EntryUpdated { entry }))
        }

        Request::DeleteEntry { key } => {
            library::delete_entry(key)
                .map_err(|error| {
                    tracing::error!(
                        "Could not delete entry {key:?}: {error}"
                    );
                })?;

            Ok(Some(Event::EntryDeleted { key }))
        }

        Request::MoveEntry { key, page_id } => {
            library::move_entry(key, page_id)
                .map_err(|error| {
                    tracing::error!(
                        "Could not move entry {key:?} to page {page_id}: {error}"
                    );
                })?;

            Ok(Some(Event::EntryMoved {
                key,
                page_id,
            }))
        }

        Request::Push => {
            library::push()
                .map_err(|error| {
                    tracing::error!(
                        "Could not push library: {error}"
                    );
                })?;

            Ok(Some(Event::Pushed))
        }
    }
}