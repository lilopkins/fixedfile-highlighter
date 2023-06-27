use std::{
    fs::File,
    io::{BufRead, BufReader, Read}, path::Path,
};

use anyhow::{Context, bail};
use base64::{engine::general_purpose, Engine};
use chrono::Local;
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

    /// The colours to output the analysed file with. This can be one of a number of inputs: a predefined preset (greyscale [default], rainbow) or; a comma separated list of hex codes.
    #[arg(short = 'c', long = "colors")]
    colors: Option<String>,

    /// Output an HTML snippet, rather than a full file
    #[arg(short = 's', long = "snippet")]
    snippet: bool,
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

    // parse colours
    let color_preset_greyscale: Vec<String> = vec!["fff".to_owned(), "ccc".to_owned()];
    let color_preset_rainbow: Vec<String> = vec!["fff".to_owned(), "f88".to_owned(), "ffc088".to_owned(), "a2ff88".to_owned(), "88f9ff".to_owned(), "a288ff".to_owned(), "ff88ba".to_owned()];

    let colors = if args.colors.is_some() {
        let c = args.colors.unwrap();
        if c.to_lowercase() == "greyscale" || c.to_lowercase() == "grayscale" {
            color_preset_greyscale
        } else if c.to_lowercase() == "rainbow" {
            color_preset_rainbow
        } else {
            let mut cs = Vec::new();
            for color in c.split(',') {
                cs.push(color.to_owned());
            }
            cs
        }
    } else {
        color_preset_greyscale
    };

    if colors.len() == 0 {
        bail!("No colours have been specified so no output can be produced!");
    }

    // parse input file into lines
    info!("Parsing input file");
    let file = File::open(&args.input_file).context("Failed to open input file.")?;
    let lines = BufReader::new(file).lines();

    // parse syntax file into vec
    info!("Parsing syntax file");
    let mut records = Vec::new();
    let mut syntax_file_reader = BufReader::new(File::open(args.syntax_file).context("Failed to open syntax file.")?);
    let mut syntax_file = String::new();
    syntax_file_reader.read_to_string(&mut syntax_file)?;
    let mut csv_reader = csv::Reader::from_reader(syntax_file.as_bytes());
    for result in csv_reader.deserialize() {
        let highlight_record: HighlightRecord = result.context("Failed to parse syntax record.")?;
        records.push(highlight_record);
    }

    // create highlighted regions and output as HTML
    info!("Creating regions and outputting");
    if !args.snippet {
        println!("<!doctype html><html>");
        println!(r#"<head><meta charset="utf8"><title>Analysis of {}</title></head>"#, Path::new(&args.input_file).file_name().unwrap().to_string_lossy());
        println!("<body>");
    }
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
        let mut color_idx = 0;
        let mut opened_tags = 0;
        for (col, chr) in line.chars().enumerate() {
            for r in &regions {
                if r.start == col {
                    let style = format!("background: #{}; color: #020202;", colors[color_idx]);
                    color_idx = (color_idx + 1) % colors.len();
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

    let mut syntax_b64 = String::new();
    general_purpose::STANDARD_NO_PAD.encode_string(syntax_file, &mut syntax_b64);
    println!(r#"Analysed at {} by <a href="https://github.com/lilopkins/fixedfile-highlighter" target="_blank" rel="noopener">fixedfile-highlighter</a> using <a href="data:text/csv;base64,{}">this syntax file</a>."#, Local::now(), syntax_b64);

    if !args.snippet {
        println!("</body></html>");
    }

    info!("Done!");
    Ok(())
}
