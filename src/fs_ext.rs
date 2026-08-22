use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, Result};

use crate::meta::types::RollbackData;

/// Any possible failure returned by `safe_write`
pub enum SafeWriteError {
    /// This error signifies any filesystem related failure, including:
    ///
    /// - Failure to move the source file
    /// - Failure to write the content into the destination
    /// - Failure to remove the backed-up source file
    WriteFailed(RollbackData),

    /// This error signifies the failure of `PathBuf::add_extension`
    DestinationDeterminationFailed,
}

/// file writing implementation that gurantees the original file contents aren't lost
/// in case a file system related operation fails.
///
/// # Panics
///
/// - If `PathBuf::add_extension` panics
pub fn safe_write<P, E, C>(path: P, ext: E, content: C) -> Result<RollbackData, SafeWriteError>
where
    P: AsRef<Path>,
    E: AsRef<str>,
    C: AsRef<[u8]>,
{
    let src = path.as_ref();
    let mut dest = src.to_owned();
    if !dest.add_extension(ext.as_ref()) {
        return Err(SafeWriteError::DestinationDeterminationFailed);
    }

    fs::rename(src, &dest).map_err(|err| {
        SafeWriteError::WriteFailed(RollbackData {
            source: src.to_owned(),
            destination: dest.clone(),
            error: Some(Box::new(err)),
        })
    })?;

    fs::write(src, content).map_err(|err| {
        SafeWriteError::WriteFailed(RollbackData {
            source: src.to_owned(),
            destination: dest.clone(),
            error: Some(Box::new(err)),
        })
    })?;

    fs::remove_file(&dest).map_err(|err| {
        SafeWriteError::WriteFailed(RollbackData {
            source: src.to_owned(),
            destination: dest.clone(),
            error: Some(Box::new(err)),
        })
    })?;

    Ok(RollbackData {
        source: src.to_owned(),
        destination: dest.clone(),
        error: None,
    })
}

/// Recursively copies all children of a directory.
///
/// # Errors
///
/// Returns `Err(Report)` if `from` is not a directory, or if either of the following fail
///
/// - `fs::read_dir`
/// - `fs_ext::copy_all`
pub fn copy_all_children<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let to = to.as_ref();

    for item in fs::read_dir(from)? {
        let item = item?;
        copy_all(item.path(), to.join(item.file_name()))?;
    }

    Ok(())
}

/// Recursively copies all files and directories.
///
/// - If `from` is a file, or a symlink to a file, it is copied directly using `fs::copy`
/// - If `from` is a directory, this function will recurse until all sub-files and directories are
///   copied.
///
/// # Errors
///
/// Returns `Err` if copying or path canonicalization fails.
pub fn copy_all<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();
    if from.is_file() || from.is_symlink() {
        fs::copy(from, to)?;
        return Ok(());
    }

    let hierarchy = DirectoryHierarchy::at(from)?;

    hierarchy.copy_to(to)?;

    Ok(())
}

#[derive(Debug)]
struct DirectoryHierarchy {
    abs_path: PathBuf,
    files: Vec<OsString>,
    dirs: Vec<DirectoryHierarchy>,
}

impl DirectoryHierarchy {
    pub fn at<P>(trunk: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let trunk = trunk.as_ref();

        let mut files = vec![];
        let mut dirs = vec![];

        for entry in fs::read_dir(trunk)?.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name();

            if file_type.is_dir()
                || (file_type.is_symlink()
                    && trunk
                        .join(&name)
                        .canonicalize()
                        .is_ok_and(|item| item.is_dir()))
            {
                dirs.push(DirectoryHierarchy::at(trunk.join(name))?)
            } else if file_type.is_file()
                || (file_type.is_symlink()
                    && trunk
                        .join(&name)
                        .canonicalize()
                        .is_ok_and(|item| item.is_file()))
            {
                files.push(name);
            }
        }

        Ok(Self {
            abs_path: trunk.to_owned(),
            files,
            dirs,
        })
    }

    pub fn copy_to<P>(&self, trunk: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let trunk = trunk.as_ref();

        fs::create_dir_all(trunk)?;
        for file in self.files.iter() {
            fs::copy(self.abs_path.join(file), trunk.join(file))?;
        }

        for dir in self.dirs.iter() {
            dir.copy_to(trunk.join(dir.dirname()?))?;
        }

        Ok(())
    }

    pub fn dirname(&self) -> Result<&OsStr> {
        self.abs_path
            .file_name()
            .ok_or_eyre("Directory has no name")
    }
}
