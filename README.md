# mdman

mdman is a command-line tool that allows you to write Unix manual pages using
simple and familiar Markdown syntax. It parses Markdown files (including GitHub
Flavored Markdown tables and YAML frontmatter) and converts them into valid
roff-formatted man pages.

## Features

- Write man pages in Markdown (with optional frontmatter for metadata)
- Converts to man-page format using roff syntax
- Supports:
  - Section and subsection headings
  - Paragraphs
  - Code blocks and inline code
  - Emphasis (italic, bold)
  - Lists (ordered and unordered)
  - Hyperlinks
  - Tables (with alignment!)
- Print to stdout for preview or debugging

## Installation

Clone and build using Cargo:

```sh
git clone https://github.com/matkrin/mdman.git
cd mdman
cargo build --release
```

The resulting binary will be in target/release/md2man.

## Usage

```sh
md2man <file.md>
```

By default, this will generate a man page with the same base filename but a .1
extension.

### Options

```
-s, --section <SECTION>  Override section number for output (e.g., 1 for general commands)
-S, --stdout             Print to stdout instead of creating a file
-o, --output <OUTPUT>    Output filename
-h, --help               Print help
-V, --version            Print version
```

### Example

```sh
md2man mytool.md
```

Generates mytool.1.

```sh
md2man mytool.md --stdout
```

Prints the man page to the terminal.

## Markdown Format

### YAML Frontmatter

Use YAML frontmatter to define required metadata for the `.TH` line:

```yaml
---
name: mytool
section: 1
date: 2025-05-24
left-footer: MyTool Manual
center-footer: MyTool v1.0
---
```

## Supported Markdown Elements

```markdown
# NAME

mytool - does things efficiently

# SYNOPSIS

mytool [OPTIONS] <INPUT>

# DESCRIPTION

_mytool_ is a small utility that...

- Lists work
  - Also nested ones

- So do **bold**, _italic_, and `inline code`

## Subsection Example

Tables work too:

| Column A | Column B |
| -------- | -------- |
| Left     | Right    |
```

## Output Format

Generates `.roff`-formatted man pages using appropriate macros:

- `.TH`, `.SH`, `.SS`, `.PP`, `.IP`, `.EX`, `.UR`, `.TS`, etc.
- Proper escaping of special characters
- Smart formatting of lists and nested structures

## Known Limitations

- No support for deeply nested formatting (e.g. bold inside italics)
- Limited error handling on malformed Markdown or YAML
- Currently assumes output section `.1` (general commands)

## Future Enhancements

- Custom output section and filename via CLI
- Support for man page cross-referencing
- Improved formatting for nested styles
- Optional preview rendering (e.g. using `man` pager directly)
