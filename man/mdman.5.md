---
name: mdman
section: 5
date: 2025-05-24
left-footer: mdman Format
center-footer: File Formats
---

# NAME

**`mdman`** - Markdown format specification for generating man pages

# DESCRIPTION

The **mdman** tool uses standard Markdown with a few conventions to generate man
pages. The expected input format includes:

- YAML frontmatter for metadata
- GitHub Flavored Markdown for content

# FRONTMATTER

Metadata at the top of the Markdown file must be formatted as a YAML block:


```yaml

---
name: mytool
section: 1
date: 2025-05-24
left-footer: MyTool Manual
center-footer: v1.0
---
```

Elements:
- `name` (required): Name of the man page
- `section` (required): Section number (1–8)
- `date` (optional): Date of last update
- `left-footer`, `center-footer` (optional): Header/footer strings

# SUPPORTED ELEMENTS

## Headings

- '#' maps to *`.SH`* (section)
- '##' maps to *`.SS`* (subsection)

## Paragraphs

Plain text separated by a blank line becomes a *`.PP`* paragraph.
Indented blocks or triple-backtick code blocks render as *`.EX`* / *`.EE`*.

## Emphasis

- `*italic*` → `\\fI...\\fP` → *italic*
- `**bold**` → `\\fB...\\fP` → **blod**
- `inline`   → `\\fC`...`\\fP` → `inline`

## Lists

Unordered lists use *`-`* and becoome *`.IP \\(bu`*.
Ordered lists use *`N.`* and become *`.IP N.`*, e.g.:

```markdown

- one
- two
    - sub 1
    - sub 2
- three
```

Resulting in:

- one
- two
    - sub 1
    - sub 2
- three

For ordered lists, you can also use the same number on all items, like so:

```markdown

1. fist
1. second
    1. sub first
    1. sub second
1. sub third
```

Result:

1. fist
1. second
    1. sub first
    1. sub second
1. sub third

## Tables

Tables are written using GitHub-Flavored Markdown syntax:

```markdown

| Column A | Column B | Column C |
|:-------- |:--------:| --------:|
| left     | center   | right    |
```

The result looks like this:

| Column A | Column B | Column C |
|:-------- |:--------:| --------:|
| left     | center   | right    |


Column alignments are respected:

- *`:---`*  = left-aligned

- *`:---:`* = center-aligned

- *`---:`*  = right-aligned

These are rendered using the roff *`.TS`*/*`.TE`* macros with allbox for boxed
tables. Each cell is wrapped in *`T{ ... T}`* for multi-line content.

Note:
- Tables must have a header row.
- Alignment rules apply to the second line of the Markdown table.
- Long cell content is supported but not automatically wrapped.

## Links

Markdown links in the form `[text](url)` are rendered using *`.UR`* / *`.UE`* blocks.
E.g.:

`[**mdman** on Github](https://github.com/matkrin/mdman)`

becomes

[mdman on Github](https://github.com/matkrin/mdman)

# SEE ALSO

mdman(1), markdown(7), man(7)
