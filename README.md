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
  - Definition lists
  - Paragraphs
  - Code blocks and inline code
  - Emphasis (italic, bold)
  - Lists (ordered and unordered)
  - Hyperlinks
  - Tables (with alignment)
- Print to stdout for preview or debugging
- Preview directly as man page

## Installation

Clone and build using Cargo:

```sh
git clone https://github.com/matkrin/mdman.git
cd mdman
cargo build --release
```

The resulting binary will be in target/release/mdman.

## Usage

```sh
mdman [OPTIONS] [<file.md>]
```

If no file is provided, Markdown is read from stdin. If no output file is
specified, output is printed to stdout.

### Options

```
-s, --section <SECTION>  Override section number for output (e.g., 1 for general commands)
-S, --stdout             Print to stdout instead of creating a file
-o, --output <OUTPUT>    Output filename (Overrides automatic naming)
-p, --pager              Preview the generated man page in a pager. (Overrides --output and --stdout)
-h, --help               Print help
-V, --version            Print version
```

### Examples

From file to stdout:

```sh
mdman doc.md --stdout
```

From stdin to stdout:

```sh
cat doc.md | mdman
```

From stdin to file:

```sh
cat doc.md | mdman --output out.5
```

### Combine with other utilities

Create HTML of the man page:

```sh
# Linux
# TODO

# macOS
mdman mytool.md --stdout | mandoc -T html > out.html
```

Use [bat](https://github.com/sharkdp/bat) as pager:

```sh
# Linux
# TODO

# macOS
mdman mytool.md --stdout | mandoc | bat
```

## Markdown Format

You can see a full description of how markdown elements get converted in [mdman(5)](/man/mdman.5.md).

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

### Some supported Markdown Elements

```markdown
# NAME

**mytool** - does things

# SYNOPSIS

**mytool** [_OPTIONS_] <_INPUT_>

# DESCRIPTION

**mytool** is a small utility that...

# OPTIONS

---
- **-h**, **--help**
  Print help message

- **-v**, **--verbose**
  Enter verbose mode
---

## Subsection Example

- Lists work
  - Also nested ones

- So do **bold**, _italic_, and `inline code`

Tables work too:

| Column A | Column B |
| -------- | -------- |
| Left     | Right    |
```

## Output Format

Generates `.roff`-formatted man pages using appropriate macros:

- `.TH`, `.SH`, `.SS`, `.PP`, `.IP`, `.EX`, `.UR`, `.TS`, etc.
- Proper escaping of special characters
- Formatting of lists and nested structures
