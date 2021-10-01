use crate::{
    errors::{Error, Result},
    metadata::RfcMetadata,
    parse_multiple,
};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Command,
};

const PR_URL: &str = "https://github.com/rust-lang/rfcs/pull/";
const TEXT_URL: &str = "https://github.com/rust-lang/rfcs/blob/master/text/";
const GIT_URL: &str = "git@github.com:rust-lang/rfcs.git";
const GIT_MAIN_BRANCH: &str = "master";
const WORKING_DIR: &str = "work";
const TEXT_DIR: &str = "text";

/// Initialises a Git repo in the working directory and pull to get it up to date.
fn init_working_repo() -> Result<()> {
    // Ignore errors (might already exist).
    let _ = fs::create_dir(WORKING_DIR);
    Command::new("git")
        .current_dir(WORKING_DIR)
        .args(&["clone", GIT_URL, "."])
        .output()?;
    Command::new("git")
        .current_dir(WORKING_DIR)
        .args(&["pull", GIT_MAIN_BRANCH, GIT_MAIN_BRANCH])
        .output()?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct GhMetadata {
    filename: String,
    path: PathBuf,
}

impl GhMetadata {
    pub fn number(&self) -> Result<u64> {
        self.filename[..4].parse().map_err(|_| Error::Parse)
    }
}

impl TryFrom<GhMetadata> for RfcMetadata {
    type Error = Error;

    fn try_from(gh: GhMetadata) -> Result<RfcMetadata> {
        let mut start_date = String::new();
        let mut feature_name = Vec::new();
        let mut issues = Vec::new();
        let file = File::open(&gh.path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if !line.starts_with("- ") {
                break;
            }

            let element: MetaTextElement = line.try_into()?;
            if element.key == "start date" {
                start_date = element.value;
            } else if element.key == "feature name" {
                feature_name = parse_multiple(&element.value);
            } else if element.key == "rust issue" || element.key == "tracking issues" {
                issues = parse_multiple(&element.value);
            }
        }

        let mut rfc = RfcMetadata::new(gh.number()?, gh.filename, start_date);
        rfc.feature_name = feature_name;
        rfc.issues = issues;
        Ok(rfc)
    }
}

struct MetaTextElement {
    key: String,
    value: String,
}

impl<'a> TryFrom<&'a str> for MetaTextElement {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<MetaTextElement> {
        let mut splits = s[1..].trim().splitn(2, ':');
        let key = splits
            .next()
            .ok_or(Error::Parse)?
            .trim()
            .to_lowercase()
            .to_owned();
        let value = splits.next().ok_or(Error::Parse)?.trim().to_owned();
        Ok(MetaTextElement { key, value })
    }
}

pub fn get_merged_rfc_data() -> Result<Vec<GhMetadata>> {
    init_working_repo()?;

    let mut text_path = PathBuf::from(WORKING_DIR);
    text_path.push(TEXT_DIR);

    let result = fs::read_dir(&text_path)?
        .filter_map(|e| e.ok())
        .filter(|p| !p.file_type().unwrap().is_dir())
        .map(|entry| GhMetadata {
            filename: entry.file_name().into_string().unwrap(),
            path: entry.path(),
        })
        .collect();

    Ok(result)
}
