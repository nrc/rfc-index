use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    str::FromStr,
};

const METADATA_VERSION: u64 = 1;
const METADATA_DIR: &str = "metadata";

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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
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
