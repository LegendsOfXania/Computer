use crate::StorageError;

use serialization::{
    decode_page,
    encode_page,
    RawPage,
};

use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};
#[derive(Debug, Clone)]
pub struct Storage {
    root: PathBuf,
}

impl Storage {
    #[inline]
    pub fn new(
        root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            root: root.into(),
        }
    }

    #[inline]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn page_path(
        &self,
        id: &str,
    ) -> Result<PathBuf, StorageError> {
        validate_page_id(id)?;

        Ok(
            self.root
                .join(id)
                .with_extension("kdl"),
        )
    }

    pub fn load_page(
        &self,
        id: &str,
    ) -> Result<RawPage, StorageError> {
        let path = self.page_path(id)?;

        let input = fs::read_to_string(path)?;

        Ok(decode_page(&input)?)
    }

    pub fn save_page(
        &self,
        page: &RawPage,
    ) -> Result<(), StorageError> {
        let path = self.page_path(page.id())?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let output = encode_page(page)?;

        fs::write(path, output)?;

        Ok(())
    }

    pub fn delete_page(
        &self,
        id: &str,
    ) -> Result<(), StorageError> {
        let path = self.page_path(id)?;

        fs::remove_file(path)?;

        Ok(())
    }

    pub fn list_pages(
        &self,
    ) -> Result<Vec<String>, StorageError> {
        let mut pages = Vec::new();

        if !self.root.exists() {
            return Ok(pages);
        }

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|extension| extension.to_str())
                != Some("kdl")
            {
                continue;
            }

            let Some(id) = path
                .file_stem()
                .and_then(|name| name.to_str())
            else {
                continue;
            };

            pages.push(id.to_owned());
        }

        pages.sort();

        Ok(pages)
    }
}

fn validate_page_id(
    id: &str,
) -> Result<(), StorageError> {
    if id.is_empty()
        || id == "."
        || id == ".."
        || id.contains('/')
        || id.contains('\\')
    {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid page id `{id}`"),
            )
            .into(),
        );
    }

    Ok(())
}