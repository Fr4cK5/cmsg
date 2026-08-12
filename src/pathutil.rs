use std::{
    env,
    path::{Path, PathBuf},
};

use eyre::Result;

pub fn normalize<P>(path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.normalize_lexically()?)
    } else {
        Ok(env::current_dir()?.join(path).normalize_lexically()?)
    }
}
