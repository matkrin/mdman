use std::fmt;
use std::io::{self, IsTerminal, Read, stdout};
use std::process::{self, Command, Stdio};
use std::{fs, io::Write, path::PathBuf};

use clap::{CommandFactory, Parser};
use man_node::{ConvertState, ManNode, convert_markdown_node};
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

    let md_content = match get_md_content(&args.file) {
        Ok(md) => md,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1)
        }
    };

    let parse_options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            gfm_table: true,
            ..Constructs::default()
        },
        ..ParseOptions::gfm()
    };

    let markdown_ast = markdown::to_mdast(&md_content, &parse_options).unwrap();
    let mut convert_state = ConvertState::new();
    let man_nodes = convert_markdown_node(&markdown_ast, &mut convert_state);

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
        _ = stdout().write_all(roff.as_bytes());
        return;
    }

    let out_path = match args.output {
        Some(output) => output,
        None => {
            let stem = args
                .file
                .as_ref()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_string_lossy();
            let base_name = PathBuf::from(stem.split('.').next().unwrap());
            base_name.with_extension(section.to_string())
        }
    };
    let mut out_file = fs::File::create(&out_path).unwrap();
    _ = out_file.write(roff.as_bytes());
}

#[derive(Debug)]
enum GetContentError {
    FileNotFound(String),
    ReadFileError(String, io::Error),
    IsTerminalError(String),
    ReadStdinError(io::Error),
}

impl fmt::Display for GetContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetContentError::FileNotFound(file) => {
                write!(f, "mdman: {}: No such file or directory", file)
            }
            GetContentError::ReadFileError(file, e) => {
                write!(f, "mdman: Could not read file {}. Error: {}", file, e)
            }
            GetContentError::IsTerminalError(h) => {
                write!(f, "mdman: Expected file or stdin\n{}", h)
            }
            GetContentError::ReadStdinError(e) => {
                write!(f, "mdman: Could not read stdin. Error: {}", e)
            }
        }
    }
}

impl std::error::Error for GetContentError {}

fn get_md_content(file_like: &Option<PathBuf>) -> Result<String, GetContentError> {
    match file_like {
        Some(file) => {
            if !file.exists() {
                return Err(GetContentError::FileNotFound(
                    file.to_string_lossy().to_string(),
                ));
            }
            match fs::read_to_string(file) {
                Ok(s) => Ok(s),
                Err(e) => Err(GetContentError::ReadFileError(
                    file.to_string_lossy().to_string(),
                    e,
                )),
            }
        }
        _ => {
            let mut stdin = io::stdin();
            if stdin.is_terminal() {
                return Err(GetContentError::IsTerminalError(
                    Args::command().render_help().to_string(),
                ));
            }
            let mut buf = String::new();
            match stdin.read_to_string(&mut buf) {
                Ok(_) => Ok(buf),
                Err(e) => Err(GetContentError::ReadStdinError(e)),
            }
        }
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
        eprintln!("mdman: Error showing man page in pager: {}", e);
        std::process::exit(1);
    }
}
