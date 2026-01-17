use std::io::Write;
use std::process::{Command, Stdio};

/// Normalize line endings and trim for consistent testing.
fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").trim().to_string()
}

#[test]
fn test_exact_roff_output_from_markdown() {
    let markdown_input = r#"---
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
- *name* (required): Name of the man page
- *section* (required): Section number (1–8)
- *date* (optional): Date of last update
- *left-footer*, *center-footer* (optional): Header/footer strings

# SUPPORTED ELEMENTS

## Headings

*#* maps to *`.SH`* (section),
*##* maps to *`.SS`* (subsection)

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

## Thematic Break

Thematic breaks (`---`) mark the start and the end of a definition list, e.g.:

```markdown

# OPTIONS

---

- **-h**, **--help**
  Print help message

- **-v**, **--verbose**
  Enter verbose mode

---
```

becomes

---

- **-h**, **--help**
  Print help message

- **-v**, **--verbose**
  Enter verbose mode

---

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

`[mdman on Github](https://github.com/matkrin/mdman)`

becomes

[mdman on Github](https://github.com/matkrin/mdman)

# SEE ALSO

mdman(1), markdown(7), man(7)
    "#;

    let expected_output = r#".TH "MDMAN" "5" "2025-05-24" "mdman Format" "File Formats"
.SH NAME
.PD
.PP
\fBmdman\fP \- Markdown format specification for generating man pages
.SH DESCRIPTION
.PD
.PP
The \fBmdman\fP tool uses standard Markdown with a few conventions to generate man
pages\&. The expected input format includes:

.RS 2
.PD 0
.IP \(bu 2
YAML frontmatter for metadata
.IP \(bu 2
GitHub Flavored Markdown for content

.RE
.SH FRONTMATTER
.PD
.PP
Metadata at the top of the Markdown file must be formatted as a YAML block:
.EX

---
name: mytool
section: 1
date: 2025-05-24
left-footer: MyTool Manual
center-footer: v1.0
---
.EE
.PD
.PP
Elements:

.RS 2
.PD 0
.IP \(bu 2
\fIname\fP (required): Name of the man page
.IP \(bu 2
\fIsection\fP (required): Section number (1–8)
.IP \(bu 2
\fIdate\fP (optional): Date of last update
.IP \(bu 2
\fIleft-footer\fP, \fIcenter-footer\fP (optional): Header/footer strings

.RE
.SH SUPPORTED ELEMENTS
.SS Headings
.PD
.PP
\fI#\fP maps to \fI.SH\fP (section),
\fI##\fP maps to \fI.SS\fP (subsection)
.SS Paragraphs
.PD
.PP
Plain text separated by a blank line becomes a \fI.PP\fP paragraph\&.
Indented blocks or triple\-backtick code blocks render as \fI.EX\fP / \fI.EE\fP\&.
.SS Emphasis

.RS 2
.PD 0
.IP \(bu 2
\fC*italic*\fP → \fC\\fI...\\fP\fP → \fIitalic\fP
.IP \(bu 2
\fC**bold**\fP → \fC\\fB...\\fP\fP → \fBblod\fP
.IP \(bu 2
\fCinline\fP   → \fC\\fC\fP\&.\&.\&.\fC\\fP\fP → \fCinline\fP

.RE
.SS Lists
.PD
.PP
Unordered lists use \fI-\fP and becoome \fI.IP \\(bu\fP\&.
Ordered lists use \fIN.\fP and become \fI.IP N.\fP, e\&.g\&.:
.EX

- one
- two
    - sub 1
    - sub 2
- three
.EE
.PD
.PP
Resulting in:

.RS 2
.PD 0
.IP \(bu 2
one
.IP \(bu 2
two
.RS 2
.PD 0
.IP \(bu 2
sub 1
.IP \(bu 2
sub 2

.RE

.IP \(bu 2
three

.RE
.PD
.PP
For ordered lists, you can also use the same number on all items, like so:
.EX

1. fist
1. second
    1. sub first
    1. sub second
1. sub third
.EE
.PD
.PP
Result:

.RS 2
.PD 0
.IP 1. 4
fist
.IP 2. 4
second
.RS 2
.PD 0
.IP 1. 4
sub first
.IP 2. 4
sub second

.RE

.IP 3. 4
sub third

.RE
.SS Thematic Break
.PD
.PP
Thematic breaks (\fC---\fP) mark the start and the end of a definition list, e\&.g\&.:
.EX

# OPTIONS

---

- **-h**, **--help**
  Print help message

- **-v**, **--verbose**
  Enter verbose mode

---
.EE
.PD
.PP
becomes
.TP
\fB-h\fP, \fB--help\fP
Print help message

.TP
\fB-v\fP, \fB--verbose\fP
Enter verbose mode

.SS Tables
.PD
.PP
Tables are written using GitHub\-Flavored Markdown syntax:
.EX

| Column A | Column B | Column C |
|:-------- |:--------:| --------:|
| left     | center   | right    |
.EE
.PD
.PP
The result looks like this:
.TS
allbox;
l c r.
T{
Column A
T}	T{
Column B
T}	T{
Column C
T}	
T{
left
T}	T{
center
T}	T{
right
T}	
.TE
.PD
.PP
Column alignments are respected:

.RS 2
.PD 0
.IP \(bu 2
\fI:---\fP  = left\-aligned
.IP \(bu 2
\fI:---:\fP = center\-aligned
.IP \(bu 2
\fI---:\fP  = right\-aligned

.RE
.PD
.PP
These are rendered using the roff \fI.TS\fP/\fI.TE\fP macros with allbox for boxed
tables\&. Each cell is wrapped in \fIT{ ... T}\fP for multi\-line content\&.
.PD
.PP
Note:

.RS 2
.PD 0
.IP \(bu 2
Tables must have a header row\&.
.IP \(bu 2
Alignment rules apply to the second line of the Markdown table\&.
.IP \(bu 2
Long cell content is supported but not automatically wrapped\&.

.RE
.SS Links
.PD
.PP
Markdown links in the form \fC[text](url)\fP are rendered using \fI.UR\fP / \fI.UE\fP blocks\&.
E\&.g\&.:
.PD
.PP
\fC[mdman on Github](https://github.com/matkrin/mdman)\fP
.PD
.PP
becomes
.PD
.PP

.UR https://github.com/matkrin/mdman
mdman on Github
.UE

.SH SEE ALSO
.PD
.PP
mdman(1), markdown(7), man(7)
    "#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_mdman"))
        .arg("--stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn mdman");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(markdown_input.as_bytes())
        .expect("Failed to write input");

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    let actual = normalize(&stdout);
    let expected = normalize(expected_output);

    assert_eq!(
        actual, expected,
        "Generated roff output did not match expected"
    );
}




