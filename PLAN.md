# what-repeats: plan

What repeats from page to page? Which lines are running headers and footers, which marks are watermarks, and what page number does each page print?

Written on 6 October 2026 before any code, as the next tool after file-checker, scan-or-text, wordbox, where-are-the-regions and what-needs-ocr. The last section says what changed once it was built.

## Purpose

A reader model sees one page at a time and takes a running header ("FINAL WELL REPORT 6507/7-10", "Page 4 of 4") for content: it gets extracted as a field, repeated in every chunk or mixed up with a value. Some OCR models drop headers and footers instead, and the printed page number goes with them. That number is what a contents page points at. "4.7 Bit record .... 3-12" means the page that prints "3-12", not the twelfth page of the file, so a page locator needs the printed labels before it can follow a contents page.

## Scope

In:

- Running lines: lines that come back at about the same place on many pages, with digits masked so "Page 3 of 60" and "Page 4 of 60" count as one line, each tagged header, footer or margin by where it sits.
- Printed page labels: what each page prints ("14", "3-12", "iv"), where, and the sequence it belongs to, with restarts reported and nothing guessed.
- /PageLabels from the catalogue when the file has them, given apart and compared with the printed labels.
- Watermarks: text drawn on many pages at the same place and size.
- Duplicate pages: the same words at the same places.

Out, on purpose:

- Scanned pages without a text layer. Their words come from OCR later, and since the rules only need words with boxes, the same tool can run on OCR output.
- Reading the contents page. That's the page locator's job, using this tool's labels.
- Any guess at what a line means beyond where it sits and how often it repeats.

## Input and output

The input is wordbox's JSON for the file: words with their boxes, line numbers and sizes. wordbox has already read the PDF, so this tool doesn't parse it again.

The output is one JSON object per file, planned as:

```json
{
  "running": [
    {"id": 0, "where": "header", "text": "FINAL WELL REPORT 6507/7-10", "pattern": "FINAL WELL REPORT #/#-#", "box": [150, 40, 450, 52], "pages": [1, 2, 3, 4]},
    {"id": 1, "where": "footer", "text": "Page 4 of 4", "pattern": "Page # of #", "box": [500, 800, 560, 810], "pages": [1, 2, 3, 4]}
  ],
  "pages": [
    {"n": 4, "label": "4", "label_box": [500, 800, 560, 810], "label_from": "running 1", "sequence": 0, "running": [0, 1], "duplicate_of": null}
  ],
  "sequences": [{"id": 0, "style": "arabic", "first_page": 1, "last_page": 4, "first_label": "1", "gaps": []}],
  "page_labels": null
}
```

## How it works

1. Lines are wordbox's lines. A line's pattern is its text with each run of digits as `#`.
2. A line is running when its pattern comes back at the same place on at least RUN_PAGES pages and at least RUN_SHARE of the document's pages. Header, footer or margin depends on which band of the page its box sits in.
3. The label is the digit or roman group in a running header or footer line that goes up by one from page to page. A label that restarts or jumps starts a new sequence, and a page with no such line gets none.
4. /PageLabels are read as a separate answer, and agreement is reported where both exist.
5. Fixed rules only, tuned on a tune split, frozen by hash, then run once on held-out files, as in the other tools.

## Data and targets

- PDFs that declare /PageLabels state each page's label, so printed labels can be checked against them where both exist. govdocs1 has plenty of government reports with roman front matter.
- A constructed set: generated multi-page documents with known headers, footers, labels in several styles (arabic, roman, "3-12" restarting by chapter), watermarks and duplicate pages.
- A hand-checked sample of real reports, including well reports from the Norwegian Offshore Directorate that carry text layers.

Targets, set before any held-out run:

- Running lines: at least 95% of the constructed set's running lines found, and at most 2% of other lines called running.
- Page labels: at least 95% of constructed pages with a printed label get the right one; on real files with /PageLabels, at least 90% agreement where a printed label is found.
- Speed: under 1 ms a page on wordbox's output.

## Changes once it was built

The constructed set came out as 240 documents in seven label styles, half tune and half held out by a hash of the document id. Every target above was met on its held-out half; README.md has the numbers.

Real files showed something the constructed set doesn't have: page numbers that change sides between odd and even pages, which never form one running line. The first version found printed labels on only 82 of the 1,116 pages of the real tune files. Labels are now also found by chaining number-like words across the header and footer bands, which brought that to 1,022, and on the 32 held-out real files 97.8% of the labels found agree with the declared ones.

Not built, or built differently: /PageLabels isn't read, since it's in the catalogue and wordbox's output doesn't carry it, so the agreement figures come from a separate check (tools/real_check.py) that reads them with PyMuPDF. The output leaves out `label_box`, `gaps` and `page_labels`. Watermarks are found from repeated large type only. Duplicate pages compare body lines, so a repeated page with a different page number still counts. The hand-checked sample of real reports hasn't been done.
