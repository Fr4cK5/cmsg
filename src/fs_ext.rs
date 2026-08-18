use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, Result};

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
