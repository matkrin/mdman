---
name: md-lsp
section: 1
date: 2025-03-29
---

# NAME
md-lsp - markdown language server

# SYNOPSIS
md-lsp [*OPTIONS*]

# OPTIONS
**-h**, **--help**
    Show help message

**-v**, **--version**
    Show the version
        
# TEST
Inline code looks like `this`. This is a link to [google](https://google.com), how to does it look like?
How does this https://man.cx look like?
A code block looks like this the following:

## Code Block
```
man mdman
```

## Bullet Lists
Bullet lists look like:
- **one**
  - sublist1
  - sublist2
    - more nested
- two
- three
- four


## Numbered Lists
Numbered lists look like:
1. **one**
2. two:
    1. sub1
    2. sub2
3. trhee
4. four
5. five

## Table

| tets | table | left | right |
|------|:-----:|:-----|------:|
|a     |b      | c    | d     |
| 1    |    2  |  3   |   4   |


```
| tets | table |
|------|-------|
|a     |b      |
```
