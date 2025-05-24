use std::io::stdout;
use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use markdown::Constructs;
use markdown::mdast::{AlignKind, Link, Table, TableCell, TableRow, Yaml};
use markdown::{
    ParseOptions,
    mdast::{
        Code, Emphasis, Heading, InlineCode, List, ListItem, Node, Paragraph, Root, Strong, Text,
    },
};
use serde::Deserialize;

mod roff;
use crate::roff::ToRoff;

// const TBL_PREPROCESSOR_INDICATOR: &str = "'\\\" t";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Markdown file to convert.
    file: PathBuf,
    /// Override section number for output (e.g., 1 for general commands).
    #[arg(short, long)]
    section: Option<u8>,
    /// Print to stdout instead of creating a file.
    #[arg(short = 'S', long)]
    stdout: bool,
    /// Output filename (Overrides automatic naming).
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let file_content = fs::read_to_string(&args.file).unwrap();
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
    if args.stdout {
        let mut stdout = stdout();
        _ = stdout.write_all(roff.as_bytes());
    } else {
        let out_path = if let Some(output) = args.output {
            output
        } else {
            let stem = args.file.file_stem().unwrap().to_string_lossy();
            let base_name = PathBuf::from(stem.split('.').next().unwrap());
            base_name.with_extension(section.to_string())
        };
        let mut out_file = fs::File::create(&out_path).unwrap();
        _ = out_file.write(roff.as_bytes());
    }
}

#[derive(Debug)]
enum ManNode {
    TitleLine(TitleLine),
    SectionHeading {
        title: String,
        children: Vec<ManNode>,
    },
    SubsectionHeading {
        title: String,
        children: Vec<ManNode>,
    },
    Paragraph {
        children: Vec<ManNode>,
    },
    Text(String),
    Bold(String),
    Italic(String),
    CodeBlock(String),
    InlineCode(String),
    BulletList {
        children: Vec<ManNode>,
    },
    NumberedList {
        children: Vec<ManNode>,
    },
    ListItem {
        children: Vec<ManNode>,
    },
    Uri {
        url: String,
        title: Option<String>,
        children: Vec<ManNode>,
    },
    Table {
        align: Vec<TableAlign>,
        children: Vec<ManNode>,
    },
    TableRow(Vec<ManNode>),
    TableCell(Vec<ManNode>),
}

#[derive(Debug, Deserialize)]
struct TitleLine {
    name: String,
    section: u8,
    date: Option<String>,
    #[serde(alias = "left-footer")]
    left_footer: Option<String>,
    #[serde(alias = "center-footer")]
    center_footer: Option<String>,
}

#[derive(Debug)]
enum TableAlign {
    Left,
    Right,
    Center,
    None,
}

impl From<&AlignKind> for TableAlign {
    fn from(value: &AlignKind) -> Self {
        match value {
            AlignKind::Left => TableAlign::Left,
            AlignKind::Right => TableAlign::Right,
            AlignKind::Center => TableAlign::Center,
            AlignKind::None => TableAlign::None,
        }
    }
}

fn convert_markdown_node(node: &Node) -> Vec<ManNode> {
    match node {
        Node::Root(Root { children, .. }) => {
            children.iter().flat_map(convert_markdown_node).collect()
        }
        Node::Yaml(Yaml { value, .. }) => {
            let title_line = serde_yaml::from_str::<TitleLine>(value).unwrap();
            vec![ManNode::TitleLine(title_line)]
        }
        Node::Heading(Heading {
            depth, children, ..
        }) => {
            let title = children.iter().map(extract_simple_text).collect();
            let heading = if *depth == 1 {
                ManNode::SectionHeading {
                    title,
                    children: vec![],
                }
            } else {
                ManNode::SubsectionHeading {
                    title,
                    children: vec![],
                }
            };
            vec![heading]
        }
        Node::Paragraph(Paragraph { children, .. }) => {
            let inlines = children.iter().flat_map(convert_markdown_node).collect();
            vec![ManNode::Paragraph { children: inlines }]
        }
        Node::Code(Code { value, .. }) => {
            vec![ManNode::CodeBlock(value.to_string())]
        }
        Node::List(List {
            children, ordered, ..
        }) => {
            let items = children.iter().flat_map(convert_markdown_node).collect();
            let man_node = if *ordered {
                ManNode::NumberedList { children: items }
            } else {
                ManNode::BulletList { children: items }
            };
            vec![man_node]
        }
        Node::ListItem(ListItem { children, .. }) => {
            let mut items = Vec::new();
            for child in children {
                let p_nodes = convert_markdown_node(child);
                for n in p_nodes {
                    match n {
                        ManNode::Paragraph { children } => items.extend(children),
                        _ => items.push(n),
                    }
                }
            }
            vec![ManNode::ListItem { children: items }]
        }
        Node::Text(Text { value, .. }) => vec![ManNode::Text(value.to_string())],
        Node::Emphasis(Emphasis { children, .. }) => {
            // TODO: Now no support for nested formatting.
            let text = children.iter().map(extract_simple_text).collect();
            vec![ManNode::Italic(text)]
        }
        Node::Strong(Strong { children, .. }) => {
            let text = children.iter().map(extract_simple_text).collect();
            vec![ManNode::Bold(text)]
        }
        Node::InlineCode(InlineCode { value, .. }) => vec![ManNode::InlineCode(value.to_string())],
        Node::Link(Link {
            children,
            url,
            title,
            ..
        }) => {
            let items = children.iter().flat_map(convert_markdown_node).collect();
            vec![ManNode::Uri {
                url: url.clone(),
                title: title.clone(),
                children: items,
            }]
        }
        Node::Table(Table {
            children, align, ..
        }) => {
            let items = children.iter().flat_map(convert_markdown_node).collect();
            let table_align = align.iter().map(Into::into).collect();
            vec![ManNode::Table {
                align: table_align,
                children: items,
            }]
        }
        Node::TableRow(TableRow { children, .. }) => {
            let items = children.iter().flat_map(convert_markdown_node).collect();
            vec![ManNode::TableRow(items)]
        }
        Node::TableCell(TableCell { children, .. }) => {
            let items = children.iter().flat_map(convert_markdown_node).collect();
            vec![ManNode::TableCell(items)]
        }
        _ => {
            dbg!(&node);
            vec![]
        }
    }
}

fn extract_simple_text(node: &Node) -> String {
    match node {
        Node::Text(Text { value, .. }) => value.to_string(),
        // For any inline element that might wrap text, simply extract its text.
        Node::Emphasis(Emphasis { children, .. }) | Node::Strong(Strong { children, .. }) => {
            children.iter().map(extract_simple_text).collect()
        }
        Node::InlineCode(InlineCode { value, .. }) => value.to_string(),
        _ => String::new(),
    }
}
