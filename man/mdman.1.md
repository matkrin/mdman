---
name: mdman
section: 1
date: 2025-05-24
left-footer: mdman Manual
center-footer: User Commands
---

# NAME

**mdman** - generate UNIX man pages from Markdown files

# SYNOPSIS

**mdman** [_OPTIONS_] _FILE_

# DESCRIPTION

**mdman** is a command-line utility that converts Markdown files into UNIX man
pages using the roff format.

Markdown syntax is parsed and rendered to support typical man page features such
as headings, paragraphs, code blocks, emphasis, lists, hyperlinks, and tables.

YAML frontmatter can be used to specify metadata like the man page name, section
number, and headers/footers.

# OPTIONS

**-o**, **--output** *FILE*
    Specify the output *FILE* manually. Overrides automatic naming.

**-S**, **--stdout**
    Print the generated roff output to stdout instead of creating a file.

**-s**, **--section** *SECTION*
    Override the output section number. Defaults to the value in YAML frontmatter,
    or 1 if none is provided.

# EXAMPLES

- Convert a Markdown file and output to mytool.1:
```sh
        $ mdman mytool.md
```

- Preview the generated output (print to stdout):

```sh
        $ mdman --stdout mytool.md
```

- Override the section number:

```sh
        $ mdman mytool.md --section 5
```

- From stdin to stdout:

```sh
        $ cat doc.md | mdman
```

- From stdin to file:

```sh
        $ cat doc.md | mdman --output out.5
```

# SEE ALSO

man(7), groff(1), markdown(5)
