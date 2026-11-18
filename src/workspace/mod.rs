//! Validated in-memory projects and private regular-file workspaces.
use std::{collections::BTreeMap, fs, io, path::Path};
use tempfile::TempDir;
use thiserror::Error;

pub const MAX_FILES: usize = 256;
pub const MAX_INPUT_BYTES: usize = 20 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("invalid project-relative path")]
    Path,
    #[error("project exceeds input limits")]
    Limit,
    #[error("main .tex file is missing")]
    Main,
    #[error("file and directory paths collide")]
    Collision,
}

pub(crate) fn validate_path(path: &str) -> Result<(), InputError> {
    if path.len() > 2048
        || path.contains(['\\', ':', '"', '\'', '`', '$', ';', '&', '|', '<', '>'])
        || path.chars().any(char::is_control)
        || path.split('/').any(|part| {
            part.len() > 255
                || part.is_empty()
                || part == "."
                || part == ".."
                || part.starts_with('-')
        })
    {
        return Err(InputError::Path);
    }
    Ok(())
}

#[derive(Debug)]
pub struct Project {
    files: BTreeMap<String, Vec<u8>>,
    main_file: String,
}

impl Project {
    /// Validate all paths and aggregate limits before writing any bytes.
    ///
    /// # Errors
    /// Rejects unsafe paths, oversized inputs, absent main files and collisions.
    pub fn new(files: BTreeMap<String, Vec<u8>>, main_file: String) -> Result<Self, InputError> {
        validate_path(&main_file)?;
        if !main_file.ends_with(".tex") || !files.contains_key(&main_file) {
            return Err(InputError::Main);
        }
        let total = files
            .values()
            .try_fold(0_usize, |sum, bytes| sum.checked_add(bytes.len()));
        if files.len() > MAX_FILES || total.is_none_or(|n| n > MAX_INPUT_BYTES) {
            return Err(InputError::Limit);
        }
        for name in files.keys() {
            validate_path(name)?;
            for (index, _) in name.match_indices('/') {
                if files.contains_key(&name[..index]) {
                    return Err(InputError::Collision);
                }
            }
        }
        Ok(Self { files, main_file })
    }

    pub(crate) fn main_file(&self) -> &str {
        &self.main_file
    }

    /// Create a fresh workspace containing only validated regular files.
    ///
    /// # Errors
    /// Returns filesystem creation/write errors; partial workspaces are removed.
    pub fn prepare(&self) -> io::Result<Workspace> {
        let directory = tempfile::Builder::new()
            .prefix("texcompiler-job-")
            .tempdir()?;
        for (name, bytes) in &self.files {
            let path = directory.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
        }
        Ok(Workspace { directory })
    }
}

/// A private workspace, removed automatically when its owner is dropped.
#[derive(Debug)]
pub struct Workspace {
    directory: TempDir,
}
impl Workspace {
    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}
