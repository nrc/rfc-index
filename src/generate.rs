use crate::errors::Result;
use std::{fs, path::PathBuf};

const OUT_DIR: &str = "target/out";
const STATIC_DIR: &str = "static";
const TEMPLATE_DIR: &str = "templates";

pub fn generate() -> Result<()> {
    // Make out dir.
    // Ignore errors (might already exist).
    let _ = fs::create_dir(OUT_DIR);

    // Copy static data. (TODO walk subdirectories)
    fs::read_dir(STATIC_DIR)?
        .filter_map(|e| e.ok())
        .filter(|p| !p.file_type().unwrap().is_dir())
        .map(|entry| {
            let mut to = PathBuf::new();
            to.push(OUT_DIR);
            to.push(entry.file_name());
            fs::copy(entry.path(), &to)?;
            Ok(())
        })
        .collect::<Result<()>>()?;

    // Copy template data (TODO do templating)
    fs::read_dir(TEMPLATE_DIR)?
        .filter_map(|e| e.ok())
        .filter(|p| !p.file_type().unwrap().is_dir())
        .map(|entry| {
            let mut to = PathBuf::new();
            to.push(OUT_DIR);
            to.push(entry.file_name());
            fs::copy(entry.path(), &to)?;
            Ok(())
        })
        .collect::<Result<()>>()?;

    Ok(())
}
