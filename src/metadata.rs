use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

const METADATA_VERSION: u64 = 1;
const METADATA_DIR: &str = "metadata";
const TAG_METADATA_FILENAME: &str = "tags.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct RfcMetadata {
    pub version: u64,
    pub number: u64,
    pub filename: String,
    pub start_date: String,
    pub feature_name: Vec<String>,
    pub issues: Vec<String>,
    pub title: Option<String>,
    pub teams: Vec<Team>,
    pub tags: Vec<String>,
}

impl RfcMetadata {
    pub fn new(number: u64, filename: String, start_date: String) -> RfcMetadata {
        RfcMetadata {
            version: METADATA_VERSION,
            number,
            filename,
            start_date,
            feature_name: Vec::new(),
            issues: Vec::new(),
            title: None,
            teams: Vec::new(),
            tags: Vec::new(),
        }
    }
}

impl PartialEq for RfcMetadata {
    fn eq(&self, other: &RfcMetadata) -> bool {
        self.number == other.number
    }
}

impl Eq for RfcMetadata {}

impl PartialOrd for RfcMetadata {
    fn partial_cmp(&self, other: &RfcMetadata) -> Option<Ordering> {
        self.number.partial_cmp(&other.number)
    }
}

impl Ord for RfcMetadata {
    fn cmp(&self, other: &RfcMetadata) -> Ordering {
        self.number.cmp(&other.number)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Team {
    Lang,
    Libs,
    Core,
    Tools,
    Compiler,
    Docs,
}

impl FromStr for Team {
    type Err = Error;

    fn from_str(s: &str) -> Result<Team> {
        match s {
            "t-lang" => Ok(Team::Lang),
            "t-libs" => Ok(Team::Libs),
            "t-core" => Ok(Team::Core),
            "t-tools" => Ok(Team::Tools),
            "t-compiler" => Ok(Team::Compiler),
            "t-docs" => Ok(Team::Docs),
            _ => Err(Error::ParseTag(s.to_owned())),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Team::Lang => write!(f, "lang"),
            Team::Libs => write!(f, "libs"),
            Team::Core => write!(f, "core"),
            Team::Tools => write!(f, "tools"),
            Team::Compiler => write!(f, "compiler"),
            Team::Docs => write!(f, "docs"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamTags {
    pub team: Team,
    pub tags: Vec<String>,
}

pub struct TagMetadata {
    pub by_tag: HashMap<String, Vec<Team>>,
    pub by_team: HashMap<Team, Vec<String>>,
}

pub fn read_tag_metadata() -> Result<TagMetadata> {
    let mut tags_path = PathBuf::from(METADATA_DIR);
    tags_path.push(TAG_METADATA_FILENAME);
    let mut file = File::open(&tags_path)?;
    let mut serialized = String::new();
    file.read_to_string(&mut serialized)?;
    let tags: Vec<TeamTags> = serde_json::from_str(&serialized)?;

    let mut by_tag = HashMap::new();
    let mut by_team = HashMap::new();

    for tt in tags {
        for t in &tt.tags {
            by_tag
                .entry(t.clone())
                .or_insert_with(|| Vec::new())
                .push(tt.team);
        }
        by_team.insert(tt.team, tt.tags);
    }

    Ok(TagMetadata { by_tag, by_team })
}

pub fn write_tag_metadata(tags: Vec<TeamTags>) -> Result<()> {
    let serialized = serde_json::to_string(&tags)?;
    let mut tags_path = PathBuf::from(METADATA_DIR);
    tags_path.push(TAG_METADATA_FILENAME);

    let mut file = File::create(&tags_path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

fn metadata_filename(number: u64) -> String {
    format!("{}/{:0>4}.json", METADATA_DIR, number)
}

pub fn save_metadata(metadata: &RfcMetadata) -> Result<()> {
    let serialized = serde_json::to_string(metadata)?;
    let mut file = File::create(metadata_filename(metadata.number))?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn open_metadata(number: u64) -> Result<RfcMetadata> {
    read_metadata(Path::new(&metadata_filename(number)))
}

fn read_metadata(path: &Path) -> Result<RfcMetadata> {
    let mut file = File::open(path)?;
    let mut serialized = String::new();
    file.read_to_string(&mut serialized)?;
    let result: RfcMetadata = serde_json::from_str(&serialized)?;

    if result.version > METADATA_VERSION {
        Err(Error::UnsupportedMetadataVersion(result.version))
    } else {
        Ok(result)
    }
}

pub fn delete_metadata(number: u64) -> Result<()> {
    fs::remove_file(metadata_filename(number))?;
    Ok(())
}

pub fn metadata_exists(number: u64) -> Result<()> {
    Path::new(&metadata_filename(number))
        .exists()
        .then(|| ())
        .ok_or(Error::FileNotFound)
}

pub fn all_metadata() -> Result<Vec<RfcMetadata>> {
    fs::read_dir(METADATA_DIR)?
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().unwrap().is_dir())
        .map(|e| read_metadata(&e.path()))
        .collect()
}

pub fn all_metadata_numbers() -> Result<Vec<u64>> {
    Ok(fs::read_dir(METADATA_DIR)?
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().unwrap().is_dir())
        .map(|e| {
            e.file_name()
                .into_string()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect())
}
