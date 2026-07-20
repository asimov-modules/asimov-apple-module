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

### `asimov-apple-calendar-emitter`

Extracts all Apple Calendar events and emits one JSON object per line (JSONL).

Each event includes:

 - `@id` (stable URN based on iCalendar UID)
 - `name` (event title)
 - `startDate`
 - `endDate`
 - `location` (omitted if not set)
 - `description` (omitted if not set)
 - `isPartOf` (calendar name)
 - `source`: "apple-calendar"

**Basic usage**
```bash
asimov-apple-calendar-emitter
```
This prints JSONL to stdout, suitable for pipelines.

**Pretty-print with jq**
```bash
asimov-apple-calendar-emitter | jq .
```

**Filter by calendar**
```bash
asimov-apple-calendar-emitter | jq 'select(.isPartOf == "Work")'
```

**Save to file**
```bash
asimov-apple-calendar-emitter > events.jsonl
```

### `asimov-apple-contacts-emitter`

Extracts all Apple Contacts people and emits one JSON object per line (JSONL).

Each contact includes:

 - `@id` (stable URN based on the Contacts person ID)
 - `name`
 - `givenName` (omitted if not set)
 - `additionalName` (omitted if not set)
 - `familyName` (omitted if not set)
 - `email` (omitted if not set)
 - `telephone` (omitted if not set)
 - `address` (omitted if not set)
 - `affiliation` (organization, omitted if not set)
 - `jobTitle` (omitted if not set)
 - `source`: "apple-contacts"

**Basic usage**
```bash
asimov-apple-contacts-emitter
```
This prints JSONL to stdout, suitable for pipelines.

**Pretty-print with jq**
```bash
asimov-apple-contacts-emitter | jq .
```

**Filter for contacts with email**
```bash
asimov-apple-contacts-emitter | jq 'select(.email != null)'
```

**Save to file**
```bash
asimov-apple-contacts-emitter > contacts.jsonl
```

## 📦 JSON Output Examples

### Notes

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

### Calendar

```json
{
  "@type": "Event",
  "@id": "urn:apple:calendar:event:ABC123-DEF456-GHI789",
  "name": "Team Standup",
  "startDate": "Wednesday, June 25, 2025 at 9:00:00 AM",
  "endDate": "Wednesday, June 25, 2025 at 9:30:00 AM",
  "location": "Conference Room B",
  "description": "Daily sync with the team.",
  "isPartOf": "Work",
  "source": "apple-calendar"
}
```

### Contacts

```json
{
  "@type": "Person",
  "@id": "urn:apple:contacts:person:12345-ABCDE",
  "name": "Jane Appleseed",
  "givenName": "Jane",
  "familyName": "Appleseed",
  "email": [
    {
      "@type": "ContactPoint",
      "name": "work",
      "value": "jane@example.com"
    }
  ],
  "telephone": [
    {
      "@type": "ContactPoint",
      "name": "mobile",
      "value": "+1 555 0100"
    }
  ],
  "affiliation": "Example Inc.",
  "jobTitle": "Product Manager",
  "source": "apple-contacts"
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
