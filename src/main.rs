use std::fmt::Write as _;
use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use markdown::Constructs;
use markdown::mdast::{Link, Yaml};
use markdown::{
    ParseOptions,
    mdast::{
        Code, Emphasis, Heading, InlineCode, List, ListItem, Node, Paragraph, Root, Strong, Text,
    },
};
use serde::Deserialize;

#[derive(Parser, Debug)]
struct Args {
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let file_content = fs::read_to_string(args.file).unwrap();
    let parse_options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::default()
        },
        ..ParseOptions::gfm()
    };
    let markdown_ast = markdown::to_mdast(&file_content, &parse_options).unwrap();
    let man_nodes = convert_markdown_node(&markdown_ast);
    // dbg!(&man_nodes);

    let roff = man_nodes.iter().map(|n| n.to_roff()).collect::<String>();
    let mut out_file = fs::File::create("./out.1").unwrap();
    _ = out_file.write(roff.as_bytes());
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
    }, // ...
}

#[derive(Debug, Deserialize)]
struct TitleLine {
    name: String,
    section: String,
    date: Option<String>,
    #[serde(alias = "left-footer")]
    left_footer: Option<String>,
    #[serde(alias = "center-footer")]
    center_footer: Option<String>,
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
            // Concatenate inline text for the heading title.
            let title = children.iter().map(|n| extract_simple_text(n)).collect();
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
            let mut inlines = Vec::new();
            for child in children {
                inlines.extend(convert_markdown_node(child));
            }
            vec![ManNode::Paragraph { children: inlines }]
        }
        Node::Code(Code { value, .. }) => {
            vec![ManNode::CodeBlock(value.to_string())]
        }
        Node::List(List {
            children,
            ordered,
            start,
            ..
        }) => {
            let mut items = Vec::new();
            for child in children {
                items.extend(convert_markdown_node(child));
            }
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
            let text = children.iter().map(|n| extract_simple_text(n)).collect();
            vec![ManNode::Italic(text)]
        }
        Node::Strong(Strong { children, .. }) => {
            let text = children.iter().map(|n| extract_simple_text(n)).collect();
            vec![ManNode::Bold(text)]
        }
        Node::InlineCode(InlineCode { value, .. }) => vec![ManNode::InlineCode(value.to_string())],
        Node::Link(Link {
            children,
            position,
            url,
            title,
        }) => {
            let mut items = Vec::new();
            for child in children {
                items.extend(convert_markdown_node(child));
            }
            vec![ManNode::Uri {
                url: url.clone(),
                title: title.clone(),
                children: items,
            }]
        }
        _ => {
            // dbg!(&node);
            vec![]
        }
    }
}

// fn convert_inline(node: &Node) -> Vec<ManNode> {
//     match node {
//         Node::Text(Text { value, .. }) => vec![ManNode::Text(value.to_string())],
//         Node::Emphasis(Emphasis { children, .. }) => {
//             // TODO: Now no support for nested formatting.
//             let text = children.iter().map(|n| extract_simple_text(n)).collect();
//             vec![ManNode::Italic(text)]
//         }
//         Node::Strong(Strong { children, .. }) => {
//             let text = children.iter().map(|n| extract_simple_text(n)).collect();
//             vec![ManNode::Bold(text)]
//         }
//         Node::InlineCode(InlineCode { value, .. }) => vec![ManNode::InlineCode(value.to_string())],
//         _ => vec![],
//     }
// }

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

trait ToRoff {
    fn to_roff(&self) -> String;
}

impl ToRoff for ManNode {
    fn to_roff(&self) -> String {
        match self {
            ManNode::TitleLine(TitleLine {
                name,
                section,
                date,
                left_footer,
                center_footer,
            }) => {
                let mut th = format!(".TH \"{}\" \"{}\"", name.to_uppercase(), section);
                if let Some(d) = date {
                    th.push_str(" \"");
                    th.push_str(d);
                    th.push('"');
                }
                if let Some(lf) = left_footer {
                    th.push_str(" \"");
                    th.push_str(lf);
                    th.push('"');
                }
                if let Some(cf) = center_footer {
                    th.push_str(" \"");
                    th.push_str(cf);
                    th.push('"');
                }
                th.push('\n');
                th
            }
            ManNode::SectionHeading { title, children } => {
                let body = children.iter().map(|n| n.to_roff()).collect::<String>();
                format!(".SH {}\n{}", title, body)
            }
            ManNode::SubsectionHeading { title, children } => {
                let body = children.iter().map(|n| n.to_roff()).collect::<String>();
                format!(".SS {}\n{}", title, body)
            }
            ManNode::Paragraph { children } => {
                let content = children.iter().map(|n| n.to_roff()).collect::<String>();
                format!(".PD\n.PP\n{}\n", content)
            }
            ManNode::Bold(text) => format!("\\fB{}\\fP", text),
            ManNode::Italic(text) => format!("\\fI{}\\fP", text),
            ManNode::InlineCode(text) => format!("\\fC{}\\fP", text),
            ManNode::CodeBlock(text) => format!(".EX\n{}\n.EE\n", text),
            ManNode::Text(text) => {
                let text = escape(text);
                if text.starts_with("\n") {
                    format!("\n.RS 8{}\n.RE", text)
                } else {
                    text.to_string()
                }
            }
            ManNode::BulletList { children } => {
                let mut content = String::new();
                for child in children {
                    content.push_str(".IP \\(bu 2\n");
                    content.push_str(&child.to_roff());
                    content.push('\n')
                }
                format!("\n.RS 2\n.PD 0\n{}\n.RE\n", content)
            }
            ManNode::NumberedList { children } => {
                let mut content = String::new();
                for (i, child) in children.iter().enumerate() {
                    _ = write!(content, ".IP {}. 4\n{}\n", i + 1, child.to_roff());
                }
                format!("\n.RS 2\n.PD 0\n{}\n.RE\n", content)
            }
            ManNode::ListItem { children } => {
                children.iter().map(|n| n.to_roff()).collect::<String>()
            }
            ManNode::Uri {
                url,
                title: _title,
                children,
            } => {
                // dbg!(&url);
                // dbg!(&_title);
                // dbg!(&children);
                let text = children.iter().map(|n| n.to_roff()).collect::<String>();
                // let url = format!("\\fI{}\\fP", url);
                format!("\n.UR {}\n{}\n.UE\n", url, text)
            }
        }
    }
}

fn escape(text: &str) -> String {
    text.replace('.', "\\&.")
}
