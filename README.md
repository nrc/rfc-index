# RFC index

A curated index of Rust RFCs.

View the RFC index at [ncameron.org/rfcs](https://www.ncameron.org/rfcs).

This project has three parts:

* a website for browsing Rust RFCs,
* [metadata](/metadata) for that website,
* [a CLI app](/src) for editing the above metadata and generating the website.

## Building and running

You'll need to get a GitHub personal access token and put it in a file called token.in in the root of this repo at compile time.

Build using the standard Cargo commands.

Commands:

```
USAGE:
    rfc-index <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add         Add new metadata for an RFC
    delete      Delete the metadata of an RFC
    generate    Generate the RFC website
    get         Query the metadata of an RFC
    help        Prints this message or the help of the given subcommand(s)
    migrate     Migrate metadata between versions
    query       Query the metadata
    scan        Scan the RFC repo for metadata
    set         Update metadata for an RFC
    stats       Emit stats about the metadata
    tag         Set/update tags on metadata
```

Use `generate` to create the website.

Use `add`, `delete`, `set`, `tag`, and `scan` to edit metadata. You can also directly edit the JSON metadata in the metadata directory.

## Contributing

Contributions are most welcome!

Contributing updates to the RFC metadata is very valuable. In particular, writing titles for RFCs, improving RFCs' tags and category (called a 'team' in the metadata and code), and adding newly merged RFCs is essential work for making this a high quality resource.

Improving the website itself is also very welcome, there are many ways we could improve the presentation and show more information, etc.

## How it works

The CLI uses Git to clone the RFC repo to get the RFC text. It uses md-book's markdown rendering and Handlebars to generate a static website. We keep metadata for each RFC and use this to generate the index of RFCs. Initial data was taken from PRs to the RFCs repo using the Octocat GitHub API library.

The metadata we use for the index is:

### Title

RFCs don't have a title, therefore we create a title for each RFC. Where there isn't a title yet, we use the RFC's filename (which is often unsatisfactory).

### Team

This is a broad category and only roughly matches the actual Rust team responsible for an RFC. An RFC can belong to multiple teams. Some RFCs have no team ('unclassified' on the website), this should be fixed.

Many RFCs' team was taken from T- labels on the RFC PR, but teams can be changed to be more useful.

### Tags

An RFC can have many tags. Tags are nested under teams using metadata in [metadata/tags.json](metadata/tags.json). Tags were mostly seeded from A- labels on RFC PRs, but again can be changed.
