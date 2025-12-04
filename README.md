# ASIMOV Apple Module

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Compatibility](https://img.shields.io/badge/rust-1.85%2B-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![Package](https://img.shields.io/crates/v/asimov-apple-module)](https://crates.io/crates/asimov-apple-module)

[ASIMOV] module for Apple devices.

## 🛠️ Prerequisites

- [Rust](https://rust-lang.org) 1.85+ (2024 edition)

## ⬇️ Installation

### Installation from Source Code

```bash
cargo install asimov-apple-module
```

## 👉 Examples

### `asimov-apple-notes-emitter`

Extracts all Apple Notes and emits one JSON object per line (JSONL).

Each note includes:

 - `@id` (stable URN)
 - `name` (title)
 - `text` (cleaned plain text converted from HTML)
 - `dateCreated`
 - `dateModified`
 - `isPartOf` (folder)
 - `account` (iCloud, On My Mac, Gmail, etc.)
 - `source`: "apple-notes"

**Basic usage**
```bash
asimov-apple-notes-emitter
```
This prints JSONL to stdout, suitable for pipelines.

**Pretty-print with jq**
```bash
asimov-apple-notes-emitter | jq .
```

**Control text wrapping**
```bash
asimov-apple-notes-emitter --wrap-width 120 | jq .
```

**Filter for a specific folder**
```bash
asimov-apple-notes-emitter | jq 'select(.isPartOf == "Work")'
```

**Save to file**
```bash
asimov-apple-notes-emitter > notes.jsonl
```

## 📦 JSON Output Example

```json
{
  "@type": "CreativeWork",
  "@id": "urn:apple:notes:note:12345-ABCDE",
  "name": "Shopping List",
  "text": "Milk\nEggs\nBread",
  "dateCreated": "2025-01-20 13:30:00 +0000",
  "dateModified": "2025-01-20 14:10:00 +0000",
  "isPartOf": "Personal",
  "account": "iCloud",
  "source": "apple-notes"
}
```

## 👨‍💻 Development

```bash
git clone https://github.com/asimov-modules/asimov-apple-module.git
```

---

[![Share on X](https://img.shields.io/badge/share%20on-x-03A9F4?logo=x)](https://x.com/intent/post?url=https://github.com/asimov-modules/asimov-apple-module&text=asimov-apple-module)
[![Share on Reddit](https://img.shields.io/badge/share%20on-reddit-red?logo=reddit)](https://reddit.com/submit?url=https://github.com/asimov-modules/asimov-apple-module&title=asimov-apple-module)
[![Share on Hacker News](https://img.shields.io/badge/share%20on-hn-orange?logo=ycombinator)](https://news.ycombinator.com/submitlink?u=https://github.com/asimov-modules/asimov-apple-module&t=asimov-apple-module)
[![Share on Facebook](https://img.shields.io/badge/share%20on-fb-1976D2?logo=facebook)](https://www.facebook.com/sharer/sharer.php?u=https://github.com/asimov-modules/asimov-apple-module)
[![Share on LinkedIn](https://img.shields.io/badge/share%20on-linkedin-3949AB?logo=linkedin)](https://www.linkedin.com/sharing/share-offsite/?url=https://github.com/asimov-modules/asimov-apple-module)

[ASIMOV]: https://github.com/asimov-platform
