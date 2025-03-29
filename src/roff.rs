use std::fmt::Write;

use crate::{ManNode, TableAlign, TitleLine};

pub trait ToRoff {
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
            ManNode::Table { align, children } => {
                let mut table = ".TS\n".to_string();
                table.push_str("allbox;\n");
                // table.push_str("box;\n");
                // table.push_str("doublebox;\n");
                let align_chars = align
                    .iter()
                    .map(|a| match a {
                        TableAlign::Left => "l",
                        TableAlign::Right => "r",
                        TableAlign::Center => "c",
                        TableAlign::None => "l",
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                table.push_str(&align_chars);
                table.push('.');
                table.push('\n');
                let text = children.iter().map(|n| n.to_roff()).collect::<String>();
                table.push_str(&text);
                table.push_str(".TE");
                table.push('\n');
                table
            }
            ManNode::TableRow(children) => {
                let text = children.iter().map(|n| n.to_roff()).collect::<String>();
                format!("{}\n", text)
            }
            ManNode::TableCell(children) => {
                let text = children.iter().map(|n| n.to_roff()).collect::<String>();
                format! {"T{{\n{}\nT}}\t", text}
            }
        }
    }
}

fn escape(text: &str) -> String {
    text.replace('.', "\\&.")
}
