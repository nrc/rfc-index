use crate::{
    errors::Result,
    github::get_merged_rfc_data,
    metadata::{open_metadata, read_tag_metadata, Team},
};
use handlebars::Handlebars;
use mdbook::utils::render_markdown;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

const OUT_DIR: &str = "target/out";
const STATIC_DIR: &str = "static";
const TEMPLATE_DIR: &str = "templates";

const INDEX_TEMPLATE: &str = "index.handlebars";
const RFC_TEMPLATE: &str = "rfc.handlebars";

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

    // Generate pages from templates + RFC data + metadata
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    let mut index_path = PathBuf::new();
    index_path.push(TEMPLATE_DIR);
    index_path.push(INDEX_TEMPLATE);
    handlebars.register_template_file("index", index_path)?;
    let mut rfc_path = PathBuf::new();
    rfc_path.push(TEMPLATE_DIR);
    rfc_path.push(RFC_TEMPLATE);
    handlebars.register_template_file("rfc", rfc_path)?;

    let mut rfc_data = get_merged_rfc_data()?;
    rfc_data.sort();
    let tag_metadata = read_tag_metadata()?;

    let mut rfcs = Vec::with_capacity(rfc_data.len());

    for rfc in rfc_data {
        let metadata = open_metadata(rfc.number)?;

        let number = format!("{:0>4}", rfc.number);
        let title = metadata
            .title
            .unwrap_or_else(|| rfc.filename[5..rfc.filename.len() - 3].to_owned());
        let url = format!("{}.html", number);

        // RFC pages
        let rfc_text = render_markdown(&rfc.text, false);
        let teams: Vec<_> = metadata.teams.iter().map(|t| t.to_string()).collect();
        let tags: Vec<_> = metadata.tags.iter().map(trim_prefix).collect();
        let html = handlebars.render(
            "rfc",
            &RfcTemplateData {
                number: number.clone(),
                title: title.clone(),
                teams: teams.join(" | "),
                tags: tags.join(" | "),
                rfc_text,
            },
        )?;
        let mut dest = PathBuf::new();
        dest.push(OUT_DIR);
        dest.push(&url);
        let mut file = File::create(dest)?;
        file.write_all(html.as_bytes())?;

        let element = IndexElement {
            number,
            title,
            url,
            teams,
            tags,
        };
        rfcs.push(element);
    }

    fn sort(input: &Vec<String>) -> Vec<String> {
        let mut out: Vec<String> = input.iter().map(trim_prefix).collect();
        out.sort();
        out
    }

    let teams = vec![
        TeamTemplateData {
            name: "core".to_owned(),
            tags: sort(&tag_metadata.by_team[&Team::Core]),
        },
        TeamTemplateData {
            name: "lang".to_owned(),
            tags: sort(&tag_metadata.by_team[&Team::Lang]),
        },
        TeamTemplateData {
            name: "libs".to_owned(),
            tags: sort(&tag_metadata.by_team[&Team::Libs]),
        },
        TeamTemplateData {
            name: "compiler".to_owned(),
            tags: sort(&tag_metadata.by_team[&Team::Compiler]),
        },
        TeamTemplateData {
            name: "tools".to_owned(),
            tags: sort(&tag_metadata.by_team[&Team::Tools]),
        },
        TeamTemplateData {
            name: "docs".to_owned(),
            tags: sort(&tag_metadata.by_team[&Team::Docs]),
        },
    ];
    let html = handlebars.render("index", &IndexTemplateData { rfcs, teams })?;
    let mut dest = PathBuf::new();
    dest.push(OUT_DIR);
    dest.push("index.html");
    let mut file = File::create(dest)?;
    file.write_all(html.as_bytes())?;

    Ok(())
}

fn trim_prefix(t: &String) -> String {
    if t.starts_with("A-") || t.starts_with("T-") {
        t[2..].to_lowercase().to_owned()
    } else {
        t.to_lowercase().to_owned()
    }
}

#[derive(Serialize, Clone)]
struct IndexTemplateData {
    rfcs: Vec<IndexElement>,
    teams: Vec<TeamTemplateData>,
}

#[derive(Serialize, Clone)]
struct TeamTemplateData {
    name: String,
    tags: Vec<String>,
}

#[derive(Serialize, Clone)]
struct IndexElement {
    number: String,
    title: String,
    url: String,
    teams: Vec<String>,
    tags: Vec<String>,
}

#[derive(Serialize, Clone)]
struct RfcTemplateData {
    number: String,
    title: String,
    teams: String,
    tags: String,
    rfc_text: String,
}
