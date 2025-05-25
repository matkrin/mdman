use std::io::{self, Read, stdout};
use std::process::{Command, Stdio};
use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use man_node::{ManNode, convert_markdown_node};
use markdown::Constructs;
use markdown::ParseOptions;

mod man_node;
mod roff;
use crate::roff::ToRoff;

// const TBL_PREPROCESSOR_INDICATOR: &str = "'\\\" t";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Markdown file to convert.
    file: Option<PathBuf>,
    /// Override section number for output (e.g., 1 for general commands).
    #[arg(short, long, conflicts_with = "pager")]
    section: Option<u8>,
    /// Print to stdout instead of creating a file.
    #[arg(short = 'S', long)]
    stdout: bool,
    /// Output filename (Overrides automatic naming).
    #[arg(short, long, conflicts_with = "stdout")]
    output: Option<PathBuf>,
    /// Preview the generated man page in a pager. (Overrides --output and --stdout).
    #[arg(short, long, conflicts_with = "output")]
    #[arg(conflicts_with = "stdout")]
    pager: bool,
}

fn main() {
    let args = Args::parse();

    let file_content = if let Some(ref file) = args.file {
        fs::read_to_string(file).unwrap()
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        buf
    };

    let parse_options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            gfm_table: true,
            ..Constructs::default()
        },
        ..ParseOptions::gfm()
    };

    let markdown_ast = markdown::to_mdast(&file_content, &parse_options).unwrap();
    let man_nodes = convert_markdown_node(&markdown_ast);

    let section = args.section.unwrap_or_else(|| {
        match man_nodes
            .iter()
            .find(|&node| matches!(node, ManNode::TitleLine(_)))
        {
            Some(ManNode::TitleLine(title_line)) => title_line.section,
            _ => 1,
        }
    });

    let roff = man_nodes.iter().map(|n| n.to_roff()).collect::<String>();

    if args.pager {
        handle_pager(&roff);
        return;
    }

    if args.stdout || args.file.is_none() {
        let mut stdout = stdout();
        _ = stdout.write_all(roff.as_bytes());
    } else {
        let out_path = if let Some(output) = args.output {
            output
        } else {
            let stem = args
                .file
                .as_ref()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_string_lossy();
            let base_name = PathBuf::from(stem.split('.').next().unwrap());
            base_name.with_extension(section.to_string())
        };
        let mut out_file = fs::File::create(&out_path).unwrap();
        _ = out_file.write(roff.as_bytes());
    }
}

fn handle_pager(roff: &str) {
    #[cfg(target_os = "macos")]
    let pager_cmd = Command::new("mandoc")
        .arg("-a")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut mandoc| {
            mandoc.stdin.as_mut().unwrap().write_all(roff.as_bytes())?;
            let output = mandoc.wait_with_output()?;
            Command::new("less")
                .stdin(Stdio::piped())
                .spawn()
                .and_then(|mut less| {
                    less.stdin.as_mut().unwrap().write_all(&output.stdout)?;
                    less.wait()?;
                    Ok(())
                })
        });

    #[cfg(target_os = "linux")]
    let pager_cmd = Command::new("man")
        .arg("-l")
        .arg("-") // read from stdin
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut man| {
            man.stdin.as_mut().unwrap().write_all(roff.as_bytes())?;
            man.wait()?;
            Ok(())
        });

    if let Err(e) = pager_cmd {
        eprintln!("Error showing man page in pager: {}", e);
        std::process::exit(1);
    }
}
