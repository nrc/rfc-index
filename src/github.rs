use crate::{
    errors::{Error, Result},
    metadata::{RfcMetadata, TagMetadata, Team, TeamTags},
    parse_multiple,
};
use octocrab::{models::pulls::PullRequest, OctocrabBuilder};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::Command,
};
use tokio::runtime::Runtime;

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
const LABEL_T_LIBS: &str = "T-libs";
const LABEL_T_LIBS_API: &str = "T-libs-api";
const LABEL_T_CORE: &str = "T-core";
const LABEL_T_COMPILER: &str = "T-compiler";
const LABEL_T_DEV_TOOLS: &str = "T-dev-tools";
const LABEL_T_RUSTDOC: &str = "T-rustdoc";
const LABEL_T_DOC: &str = "T-doc";

async fn get_pr(number: u64) -> Result<PullRequest> {
    let pr = OctocrabBuilder::new()
        .personal_token(include_str!("../token.in").to_owned())
        .build()
        .unwrap()
        .pulls("rust-lang", "rfcs")
        .get(number)
        .await?;
    Ok(pr)
}

pub struct UpdateOptions {
    pub tags: bool,
    pub teams: bool,
}

impl UpdateOptions {
    pub fn all() -> UpdateOptions {
        UpdateOptions {
            tags: true,
            teams: true,
        }
    }
}

pub fn update_from_pr(
    metadata: &mut RfcMetadata,
    tag_metadata: &TagMetadata,
    opts: UpdateOptions,
) -> Result<()> {
    if !opts.tags && !opts.teams {
        return Ok(());
    }

    Runtime::new().unwrap().block_on(async {
        let pr = get_pr(metadata.number).await?;

        match &pr.labels {
            Some(l) => {
                if opts.teams {
                    // Teams
                    let teams = l.iter().filter_map(|l| match &*l.name {
                        LABEL_T_LANG => Some(Team::Lang),
                        LABEL_T_LIBS | LABEL_T_LIBS_API => Some(Team::Libs),
                        LABEL_T_CORE => Some(Team::Core),
                        LABEL_T_COMPILER => Some(Team::Compiler),
                        LABEL_T_DEV_TOOLS | LABEL_T_RUSTDOC | LABEL_T_CARGO => Some(Team::Tools),
                        LABEL_T_DOC => Some(Team::Docs),
                        _ => None,
                    });

                    for team in teams {
                        if !metadata.teams.contains(&team) {
                            metadata.teams.push(team);
                        }
                    }
                }

                if opts.tags {
                    // Tags
                    for l in l {
                        let l = &l.name;
                        if tag_metadata.by_tag.contains_key(l) {
                            let l = l.to_owned();
                            if !metadata.tags.contains(&l) {
                                metadata.tags.push(l);
                            }
                        }
                    }
                }
            }
            None => return Err(Error::GitHub), // format!("No labels for PR {}", metadata.number),
        }

        Ok(())
    })
}

pub fn init_tag_metadata() -> Result<Vec<TeamTags>> {
    init_working_repo()?;

    let mut text_path = PathBuf::from(WORKING_DIR);
    text_path.push(TEXT_DIR);

    let numbers = fs::read_dir(&text_path)?
        .filter_map(|e| e.ok())
        .filter(|p| !p.file_type().unwrap().is_dir())
        .map(|entry| {
            let filename = entry.file_name().into_string().unwrap();
            let number = rfc_number(&filename)?;
            Ok(number)
        })
        .collect::<Result<Vec<u64>>>()?;

    Runtime::new().unwrap().block_on(async {
        let mut result = HashMap::new();
        result.insert(Team::Lang, Vec::new());
        result.insert(Team::Libs, Vec::new());
        result.insert(Team::Core, Vec::new());
        result.insert(Team::Tools, Vec::new());
        result.insert(Team::Compiler, Vec::new());
        result.insert(Team::Docs, Vec::new());

        enum TeamOrTag {
            Team(Team),
            Tag(String),
        }

        for n in numbers {
            let pr = get_pr(n).await?;
            match &pr.labels {
                Some(l) => {
                    let (teams, tags): (Vec<TeamOrTag>, _) = l
                        .iter()
                        .filter(|l| l.name.starts_with("T-") || l.name.starts_with("A-"))
                        .map(|l| match &*l.name {
                            LABEL_T_LANG => TeamOrTag::Team(Team::Lang),
                            LABEL_T_LIBS_API => TeamOrTag::Team(Team::Libs),
                            LABEL_T_CORE => TeamOrTag::Team(Team::Core),
                            LABEL_T_COMPILER => TeamOrTag::Team(Team::Compiler),
                            LABEL_T_DEV_TOOLS | LABEL_T_RUSTDOC | LABEL_T_CARGO => {
                                TeamOrTag::Team(Team::Tools)
                            }
                            LABEL_T_DOC => TeamOrTag::Team(Team::Docs),
                            s => TeamOrTag::Tag(s.to_owned()),
                        })
                        .partition(|tt| matches!(tt, TeamOrTag::Team(_)));

                    if tags.is_empty() {
                        continue;
                    }

                    let teams: Vec<_> = teams
                        .into_iter()
                        .map(|t| match t {
                            TeamOrTag::Team(t) => t,
                            _ => unreachable!(),
                        })
                        .collect();
                    let tags: Vec<_> = tags
                        .into_iter()
                        .map(|t| match t {
                            TeamOrTag::Tag(t) => t,
                            _ => unreachable!(),
                        })
                        .collect();

                    match teams.len() {
                        0 => println!("No teams: [{}]", tags.join(", ")),
                        1 => {
                            for t in tags {
                                let v = &mut result.get_mut(&teams[0]).unwrap();
                                if !v.contains(&t) {
                                    v.push(t);
                                }
                            }
                        }
                        _ => println!("Multiple teams ({:?}): [{}]", teams, tags.join(", ")),
                    }
                }
                None => return Err(Error::GitHub), // format!("No labels for PR {}", metadata.number),
            }
        }

        Ok(result
            .into_iter()
            .map(|(team, tags)| TeamTags { team, tags })
            .collect())
    })
}
