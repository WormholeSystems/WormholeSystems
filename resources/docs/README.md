# Documentation

These Markdown files power the in-app **Documentation** at `/documentation`. Anyone can contribute — add or edit a file and it shows up automatically.

## How it's organised

```
resources/docs/
  01-getting-started/        ← a category (folder)
    01-overview.md           ← a page
    02-connecting-your-character.md
  02-signatures/
    ...
```

- **Folders are categories**, **files are pages**.
- The `NN-` number prefix only controls **ordering** in the sidebar. It's stripped from the URL.
- The folder/file name (minus the prefix) becomes the **URL slug**. For example `01-getting-started/02-connecting-your-character.md` is served at `/documentation/getting-started/connecting-your-character`.

## Frontmatter

Each page starts with a small YAML frontmatter block:

```markdown
---
title: Connecting your character
---

# Connecting your character

Body content in **Markdown**…
```

- `title` — the label shown in the sidebar and as the page heading. Optional; if omitted, the first `# Heading` (or the filename) is used.
- `category` — optional override for the category label; defaults to the humanised folder name.

## Writing content

- Standard **GitHub-flavoured Markdown**: headings, lists, tables, code, blockquotes, and links all work.
- Link between pages with absolute paths, e.g. `[Auto-mapping](/documentation/getting-started/auto-mapping)`.
- Raw HTML is stripped for safety — stick to Markdown.

## Adding a new page

1. Drop a `NN-your-page.md` file into the right category folder (or create a new `NN-category/` folder).
2. Add the frontmatter `title`.
3. Write Markdown. That's it — the sidebar, ordering, and prev/next links update on their own.
