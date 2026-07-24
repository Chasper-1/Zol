# .rdb Format (Read Book)

> **Status:** design phase  
> **Version:** 0.1 (draft)

---

## Description

**.rdb** is a universal binary container for text with optional markup.
Not tied to zoll: it can store any text (zoll, md, html, plain) with or without markup preservation.

**Key features:**
- Two-level compression (semantic dictionary + zstd) — 20–30% more compact than raw zstd
- Pointers for styles and interactive elements (terms, links, spoilers, formulas)
- Read without parsing — deserialization only
- Instant open — tokens are already ready, no parser needed
- Export to MD, HTML, PDF, TXT without re-parsing the source
- Embedding images and translations (multi-language) in a single file

---

## Compression Architecture

```
┌─ Source text ─────────────────────────────┐
│  «The quick brown fox jumps over the lazy  │
│   dog. The quick brown fox jumps again.»   │
└────────────────────────────────────────────┘
                      │
                      ▼
┌─ Level 1: Semantic Dictionary ───────────┐
│  Dictionary: {1: quick, 2: brown,          │
│              3: fox, 4: jumps}             │
│  Text: 1 2 3 4 5 6 7 1 2 3 4              │
│  (frequent/long words → index)             │
└────────────────────────────────────────────┘
                      │
                      ▼
┌─ Level 2: zstd ──────────────────────────┐
│  Index byte stream → zstd block            │
└────────────────────────────────────────────┘
```

The two levels work with **different types of redundancy**:

| Level | Redundancy type | Example |
|---|---|---|
| Dictionary | Lexical | "quick" ×50 |
| zstd | Structural | Repeated `1 2 3 4` pattern |

They don't interfere — they complement each other.

---

## VarInt Encoding for Dictionary Indices

Word indices are encoded with a variable number of bytes — the more frequent
the word, the shorter the index.

| Index range | First byte | Capacity |
|---|---|---|
| 0–127 | `0xxxxxxx` — 1 byte | 128 most frequent words |
| 128–16,511 | `10xxxxxx xxxxxxxx` — 2 bytes | ~16K words |
| 16,512–2,097,151 | `110xxxxx xxxxxxxxx xxxxxxxx` — 3 bytes | ~2M words |

Short words (1–4 bytes) are not added to the dictionary — replacing them with
an index yields no gain. The compilation algorithm evaluates each word's "weight"
(frequency × length) and decides whether to include it automatically.

---

## Pointers (Styles, Interactivity)

Markup is stored separately from the text as pointers:

```
(line: u32, token_on_line: u16, pointer_type: u8, data: ...)
```

**Pointer types:**

| Type | Purpose | Data |
|---|---|---|
| `Bold` / `Italic` / etc. | Markup styles | — |
| `Term` | Term definition | Term ID in dictionary |
| `Spoiler` | Hidden text | — |
| `Link` | Hyperlink | URL or document ID |
| `Footnote` | Footnote | Footnote ID |
| `Formula` | Mathematical formula | Formula ID |
| `Image` | Embedded image | Offset + size in container |

Pointers **do not participate in semantic compression** — they are compact
enough on their own. zstd will, however, compress repeated pointer sequences.

Thanks to pointers, reading .rdb **does not require the zoll parser** —
only deserialization.

---

## File Structure

```
┌─────────────────────────────┐
│  Magic: "rdb\0" (4 bytes)   │
├─────────────────────────────┤
│  Header (metadata)           │
│  - author, title, date       │
│  - language (multi-language) │
├─────────────────────────────┤
│  Dictionary                  │
│  - list of unique words      │
│  - frequency statistics      │
├─────────────────────────────┤
│  Text (word indices)         │
│  - zstd-compressed stream    │
├─────────────────────────────┤
│  Pointers (optional)         │
│  - styles, links, terms      │
│  - zstd-compressed stream    │
├─────────────────────────────┤
│  Resources (optional)        │
│  - images, fonts             │
│  - translations (multi-lang) │
│  - zstd-compressed stream    │
└─────────────────────────────┘
```

---

## Compilation Statistics

When building an .rdb file, a report is printed. It allows the author to see
which words yield the greatest compression gain.

```
.zoll → .rdb compression
━━━━━━━━━━━━━━━━━━━━━━━
Original .zoll:               1 240 000 B
Plain text (markers stripped):  980 000 B
After dictionary:               690 000 B
After zstd:                     210 000 B

Heaviest words:
  eloquent        (×312)  8 736 B →  312 B   (-96%)
  extraordinary   (×48)   1 296 B →   48 B   (-96%)
  comprehensive   (×211)  4 433 B →  211 B   (-95%)
  ...

Savings: 83% relative to .zoll
         56% relative to raw zstd

Compilation time: 127 ms
```

---

## Conversion

Any format can be exported from .rdb **without re-parsing the source**:

```
       ┌──→ .zoll    (restore markers from pointers)
       │
.rdb ──┼──→ .md      (pointers → md syntax)
       │
       ├──→ .html     (pointers → HTML tags)
       │
       ├──→ .pdf      (direct page rendering)
       │
       └──→ .txt      (plain text, no markup)
```

---

## Content Protection

The format does not encrypt data, but makes casual extraction difficult:

- Text is stored as dictionary indices, not in human-readable form
- Style pointers are binary offsets without public documentation
- Optional obfuscation (shuffling sections) on top of the structure
- Format specification may not be publicly available

**This is not DRM.** It is "honest protection" — the file can be read through
an official reader, but copying the markup or extracting the text with
third-party tools is sufficiently difficult.

---

## Implementation Status

- [x] Two-level compression concept (dictionary + zstd)
- [x] VarInt index encoding
- [x] Alignment with zoll incremental parser architecture
- [ ] Basic container (header + dictionary + text)
- [ ] Style pointers for zoll markup
- [ ] Image embedding
- [ ] Multi-language sections
- [ ] Compilation statistics
- [ ] Export to MD / HTML / PDF
- [ ] Open specification as a standalone project

Implementation will begin after the core zoll incremental parser is complete.

---

## Comparison with Existing Formats

| | PDF | EPUB | DOCX | **.rdb** |
|---|---|---|---|---|
| Size | Large | Medium | Large | **Smallest** |
| Compression | None | ZIP | ZIP | **zstd + dictionary** |
| Interactivity | None | Weak (JS) | Partial | **Native** |
| Open speed | Medium | Slow (XML) | Medium | **Instant** |
| Markup | Visual | Logical | Logical | **Pointers** |
| Read without parser | Yes | Yes | Yes | **Yes** |
| Resource embedding | Yes | Yes | Yes | **Yes** |
| Open format | Yes | Yes | No | **Yes (optional)** |
