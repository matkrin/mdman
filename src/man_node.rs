use markdown::mdast::{
    AlignKind, Code, Emphasis, Heading, InlineCode, Link, List, ListItem, Node, Paragraph, Root,
    Strong, Table, TableCell, TableRow, Text, Yaml,
};
use serde::Deserialize;

#[derive(Debug)]
pub enum ManNode {
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
    DefinitionList {
        children: Vec<ManNode>,
    },
}

#[derive(Debug, Deserialize)]
pub struct TitleLine {
    pub name: String,
    pub section: u8,
    pub date: Option<String>,
    #[serde(alias = "left-footer")]
    pub left_footer: Option<String>,
    #[serde(alias = "center-footer")]
    pub center_footer: Option<String>,
}

#[derive(Debug)]
pub enum TableAlign {
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

pub struct ConvertState {
    in_definition_list: bool,
}
impl ConvertState {
    pub fn new() -> Self {
        Self {
            in_definition_list: false,
        }
    }
    fn toggle_in_definition_list(&mut self) {
        self.in_definition_list = !self.in_definition_list
    }
}

pub fn convert_markdown_node(node: &Node, state: &mut ConvertState) -> Vec<ManNode> {
    match node {
        Node::Root(Root { children, .. }) => children
            .iter()
            .flat_map(|x| convert_markdown_node(x, state))
            .collect(),
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
            let inlines = children
                .iter()
                .flat_map(|x| convert_markdown_node(x, state))
                .collect();
            vec![ManNode::Paragraph { children: inlines }]
        }
        Node::Code(Code { value, .. }) => {
            vec![ManNode::CodeBlock(value.to_string())]
        }
        Node::List(List {
            children, ordered, ..
        }) => {
            let items = children
                .iter()
                .flat_map(|x| convert_markdown_node(x, state))
                .collect();

            let man_node = match (ordered, state.in_definition_list) {
                (true, _) => ManNode::NumberedList { children: items },
                (false, true) => ManNode::DefinitionList { children: items },
                (false, false) => ManNode::BulletList { children: items },
            };
            vec![man_node]
        }
        Node::ListItem(ListItem { children, .. }) => {
            let mut items = Vec::new();
            for child in children {
                let p_nodes = convert_markdown_node(child, state);
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
            let items = children
                .iter()
                .flat_map(|x| convert_markdown_node(x, state))
                .collect();
            vec![ManNode::Uri {
                url: url.clone(),
                title: title.clone(),
                children: items,
            }]
        }
        Node::Table(Table {
            children, align, ..
        }) => {
            let items = children
                .iter()
                .flat_map(|x| convert_markdown_node(x, state))
                .collect();
            let table_align = align.iter().map(Into::into).collect();
            vec![ManNode::Table {
                align: table_align,
                children: items,
            }]
        }
        Node::TableRow(TableRow { children, .. }) => {
            let items = children
                .iter()
                .flat_map(|x| convert_markdown_node(x, state))
                .collect();
            vec![ManNode::TableRow(items)]
        }
        Node::TableCell(TableCell { children, .. }) => {
            let items = children
                .iter()
                .flat_map(|x| convert_markdown_node(x, state))
                .collect();
            vec![ManNode::TableCell(items)]
        }
        Node::ThematicBreak(_) => {
            state.toggle_in_definition_list();
            vec![]
        }
        _ => {
            // dbg!(&node);
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
