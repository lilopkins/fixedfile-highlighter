use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use clap::Parser;
use log::{info, warn};
use regex::Regex;

/// Highlight parts of a file given a syntax.
///
/// We parse over a syntax CSV, expecting a header row containing `start,length,name,condition', where:
///   `start` is the 1-based start column of the character to highlight
///   `length` is the number of columns of this field
///   `name` is the human readable name for this field
///   `condition` (optional) is a regex to restrict this rule applying except to lines that match the regex.
/// Rules are applied top-to-bottom.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The input file to process
    #[arg(index = 1)]
    input_file: String,

    /// The syntax file to use
    #[arg(index = 2)]
    syntax_file: String,
}

#[derive(Debug, serde::Deserialize)]
struct HighlightRecord {
    start: usize,
    length: usize,
    name: String,
    condition: Option<String>,
}

#[derive(Debug)]
struct HighlightRegion {
    start: usize,
    end: usize,
    name: String,
    applied: bool,
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_custom_env("LOG");
    let args = Args::parse();

    // parse input file into lines
    info!("Parsing input file");
    let file = File::open(args.input_file).context("Failed to open input file.")?;
    let lines = BufReader::new(file).lines();

    // parse syntax file into vec
    info!("Parsing syntax file");
    let mut records = Vec::new();
    let mut csv_reader = csv::Reader::from_reader(BufReader::new(
        File::open(args.syntax_file).context("Failed to open syntax file.")?,
    ));
    for result in csv_reader.deserialize() {
        let highlight_record: HighlightRecord = result.context("Failed to parse syntax record.")?;
        records.push(highlight_record);
    }

    // create highlighted regions and output as HTML
    info!("Creating regions and outputting");
    println!("<pre>");
    for (idx, line) in lines.enumerate() {
        let line = line.context("Failed to read line from input file.")?;
        let mut regions = Vec::new();

        // produce regions
        for record in &records {
            let apply_record_to_this_line = if record.condition.is_some() {
                let re = Regex::new(&record.condition.clone().unwrap())
                    .context("Failed to parse condition regex.")?;
                re.is_match(&line)
            } else {
                true
            };

            if apply_record_to_this_line {
                regions.push(HighlightRegion {
                    start: record.start - 1,
                    end: record.start + record.length - 1,
                    name: record.name.clone(),
                    applied: false,
                })
            }
        }

        // output line
        let mut alt = false;
        let mut opened_tags = 0;
        for (col, chr) in line.chars().enumerate() {
            for r in &regions {
                if r.start == col {
                    let style = if alt { "background: #ccc;" } else { "background: #fff;" };
                    alt = !alt;
                    opened_tags += 1;
                    print!(r#"<abbr title="{}" style="{}">"#, r.name, style);
                }
            }
            print!("{}", chr);
            for r in &mut regions {
                if r.end == col + 1 {
                    print!("</abbr>");
                    opened_tags -= 1;
                    r.applied = true;
                }
            }
        }
        
        if opened_tags != 0 {
            warn!("Line {} was not long enough to fit the matching regions.", idx);
            for _ in 0..opened_tags {
                print!("</abbr>");
            }
        }

        println!();

        for r in regions {
            if r.applied == false {
                warn!("Failed to highlight rule {} on line {}!", r.name, idx);
            }
        }
    }
    println!("</pre>");
    
    info!("Done!");
    Ok(())
}
