use crate::{
    errors::{Error, Result},
    metadata::{RfcMetadata, Team},
    parse_multiple,
};
use octocrab::models::pulls::PullRequest;
use std::{
    cmp::Ordering,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::Command,
};
use tokio::runtime::Runtime;

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

#[derive(Debug, Clone)]
pub struct GhData {
    pub number: u64,
    pub filename: String,
    pub text: String,
}

impl PartialEq for GhData {
    fn eq(&self, other: &GhData) -> bool {
        self.number == other.number
    }
}

impl Eq for GhData {}

impl PartialOrd for GhData {
    fn partial_cmp(&self, other: &GhData) -> Option<Ordering> {
        self.number.partial_cmp(&other.number)
    }
}

impl Ord for GhData {
    fn cmp(&self, other: &GhData) -> Ordering {
        self.number.cmp(&other.number)
    }
}

impl GhMetadata {
    pub fn number(&self) -> Result<u64> {
        rfc_number(&self.filename)
    }
}

fn rfc_number(filename: &str) -> Result<u64> {
    filename[..4].parse().map_err(|_| Error::Parse)
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

pub fn get_merged_rfc_metadata() -> Result<Vec<GhMetadata>> {
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

pub fn get_merged_rfc_data() -> Result<Vec<GhData>> {
    init_working_repo()?;

    let mut text_path = PathBuf::from(WORKING_DIR);
    text_path.push(TEXT_DIR);

    fs::read_dir(&text_path)?
        .filter_map(|e| e.ok())
        .filter(|p| !p.file_type().unwrap().is_dir())
        .map(|entry| {
            let filename = entry.file_name().into_string().unwrap();
            let number = rfc_number(&filename)?;
            let mut file = File::open(&entry.path())?;
            let mut text = String::new();
            file.read_to_string(&mut text)?;

            Ok(GhData {
                number,
                filename,
                text,
            })
        })
        .collect()
}

const LABEL_T_LANG: &str = "T-lang";
const LABEL_T_CARGO: &str = "T-cargo";
const LABEL_T_LIBS_API: &str = "T-libs-api";
const LABEL_T_CORE: &str = "T-core";

async fn get_pr(number: u64) -> Result<PullRequest> {
    let pr = octocrab::instance()
        .pulls("rust-lang", "rfcs")
        .get(number)
        .await?;
    Ok(pr)
}

async fn any_label<T>(number: u64, f: impl Fn(&str) -> Option<T>) -> Result<Option<T>> {
    Ok(get_pr(number)
        .await?
        .labels
        .unwrap_or(Vec::new())
        .iter()
        .filter_map(|l| f(&l.name))
        .next())
}

pub fn team(number: u64) -> Result<Team> {
    Runtime::new().unwrap().block_on(async {
        any_label(number, |l| match l {
            LABEL_T_LANG => Some(Team::Lang),
            LABEL_T_CARGO => Some(Team::Cargo),
            LABEL_T_LIBS_API => Some(Team::Libs),
            LABEL_T_CORE => Some(Team::Core),
            _ => None,
        })
        .await?
        .ok_or(Error::MissingMetadata)
    })
}
