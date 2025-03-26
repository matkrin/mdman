use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use markdown::{
    ParseOptions,
    mdast::{Code, Emphasis, Heading, InlineCode, Node, Paragraph, Root, Strong, Text},
};

#[derive(Parser, Debug)]
struct Args {
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let file_content = fs::read_to_string(args.file).unwrap();
    let markdown_ast = markdown::to_mdast(&file_content, &ParseOptions::default()).unwrap();
    let man_nodes = convert_markdown_node(&markdown_ast);
    dbg!(&man_nodes);

    let roff = man_nodes.iter().map(|n| n.to_roff()).collect::<String>();
    let mut out_file = fs::File::create("./out.1").unwrap();
    _ = out_file.write(roff.as_bytes());
}

#[derive(Debug)]
enum ManNode {
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
    // ...
}

fn convert_markdown_node(node: &Node) -> Vec<ManNode> {
    match node {
        Node::Root(Root { children, .. }) => {
            children.iter().flat_map(convert_markdown_node).collect()
        }
        Node::Heading(Heading {
            depth, children, ..
        }) => {
            dbg!(&depth);
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
                inlines.extend(convert_inline(child));
            }
            vec![ManNode::Paragraph { children: inlines }]
        }
        Node::Code(Code { value, .. }) => {
            vec![ManNode::CodeBlock(value.to_string())]
        }
        _ => vec![],
    }
}

fn convert_inline(node: &Node) -> Vec<ManNode> {
    match node {
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
        _ => vec![],
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

trait ToRoff {
    fn to_roff(&self) -> String;
}

impl ToRoff for ManNode {
    fn to_roff(&self) -> String {
        match self {
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
                format!(".PP\n{}\n", content)
            }
            ManNode::Bold(text) => format!("\\fB{}\\fP", text),
            ManNode::Italic(text) => format!("\\fI{}\\fP", text),
            ManNode::InlineCode(text) => format!("\\fC{}\\fP", text),
            ManNode::CodeBlock(text) => format!(".EX\n{}\n.EE\n", text),
            ManNode::Text(text) => {
                if text.starts_with("\n") {
                    format!("\n.RS 8{}\n.RE", text)
                } else {
                    text.to_string()
                }
            }
        }
    }
}
