use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

const METADATA_VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct RfcMetadata {
    pub version: u64,
    pub number: u64,
    pub filename: String,
    pub start_date: String,
    pub merge_date: Option<String>,
    pub feature_name: Vec<String>,
    pub issues: Vec<String>,
    pub title: Option<String>,
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Tag {
    Team(Team),
    Topic(Topic),
    Custom(String),
}

// TODO should be custom?
#[derive(Serialize, Deserialize, Debug)]
pub enum Topic {
    Traits,
    TraitObjects,
    Dsts,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Team {
    Lang,
    Libs,
    Cargo,
    Core,
    Tools,
}

fn metadata_filename(number: u64) -> String {
    format!("metadata/{:0>4}.json", number)
}

pub fn open_metadata(number: u64) -> Result<RfcMetadata> {
    let mut file = File::open(metadata_filename(number))?;
    let mut serialized = String::new();
    file.read_to_string(&mut serialized)?;
    let result: RfcMetadata = serde_json::from_str(&serialized)?;

    if result.version > METADATA_VERSION {
        Err(Error::UnsupportedMetadataVersion(result.version))
    } else {
        Ok(result)
    }
}
