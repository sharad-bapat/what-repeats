# what-repeats: plan

What repeats from page to page? Which lines are running headers and footers, which marks are watermarks, and what page number does each page print?

Draft, 6 October 2026, for review before any code. The sixth tool in the series, after file-checker, scan-or-text, wordbox, where-are-the-regions and what-needs-ocr.

## Why

A reader model sees one page at a time and treats a running header ("CONOCO FINAL WELL REPORT 6507/7-10", "Page 4 of 4") as content: it gets extracted as a field, repeated in every chunk, or confused with a value. olmOCR drops headers and footers outright, which loses the printed page number. And the printed page number is what a table of contents points at: "4.7 Bit record .... 3-12" means the page that prints "3-12", not the 12th page of the file. A page locator needs that map before it can follow a contents page.

## Scope

In:

- Running lines: text lines that recur at about the same place on many pages, with digits masked so "Page 3 of 60" and "Page 4 of 60" count as the same line. Each tagged header, footer or margin by where it sits.
- Printed page labels: the number or label each page prints ("14", "3-12", "iv", "A-2"), where it sits, and the sequence it belongs to, with gaps and restarts reported, not guessed.
- /PageLabels from the document catalogue when present, given apart and compared with the printed labels.
- Watermarks: text or images drawn on many pages at the same place and size, or flagged by where-are-the-regions as drawn under the text.
- Exact duplicate pages: the same words at the same places.

Out, on purpose:

- Scanned pages without a text layer. Their lines come later from OCR; the rules take words with boxes from whatever source, so the same tool can run on OCR output.
- Reading the table of contents. That belongs to the page locator, which will use this tool's label map.
- Any guess at what a line means beyond its place and its repetition.

## Input and output

Input: wordbox's JSON for the file (words with boxes, lines and sizes), so the PDF is parsed once. Decided 6 October 2026: the tools in this series are parts of one engine and build on each other's output; none re-parses what an earlier tool already gives. what-repeats has no PDF parser of its own, and because it only needs words with boxes, it can run on OCR words in the same shape.

Output, one JSON object per file:

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

## How it will work

1. Lines from wordbox's words (its line numbers and boxes). A line's key is its text with each run of digits replaced by `#`, plus its box rounded to a grid of a few points.
2. A line is running when its key, or its position with a similar key, recurs on at least RUN_PAGES pages and at least RUN_SHARE of a run of consecutive pages. Header, footer or margin by the box's place in the top, bottom or side band of the page.
3. Page labels: within running lines in the header and footer bands, the digit or roman group that changes from page to page by +1 is the label. Labels that restart or jump start a new sequence. Pages with no such line get none.
4. /PageLabels read as a separate answer; where both exist, agreement is reported.
5. Fixed rules only, tuned on a dev split, frozen by hash, then run once on held-out files, as before.

## Ground truth and tests

- PDFs with /PageLabels: the label of each page is stated by the file, so printed labels can be checked against it where both exist. govdocs1 has many government reports with front matter in roman numerals.
- A constructed set: generated multi-page documents with known headers, footers, labels in several styles (arabic, roman, "3-12", restarts per chapter), watermarks and duplicate pages.
- A hand-checked sample of real reports, including Sodir completion reports with text layers.

Targets, set before the held-out run:

- Running lines: at least 95% of constructed running lines found, and at most 2% of other lines called running.
- Page labels: at least 95% of pages with a printed label get the right one on the constructed set; on real files with /PageLabels, at least 90% agreement where the printed label exists.
- Speed: under 1 ms a page on wordbox's output.

## Open questions

1. Name: "what-repeats" asks the question; "where-are-the-page-numbers" names the main use.
