# what-repeats

What repeats from page to page? Running headers, footers and margin lines, the label each page prints and the sequences those labels form, watermarks, and pages whose body repeats another's.

It reads wordbox's JSON, so it has no PDF parser of its own and runs on any words with boxes, OCR words included. Part of a series that builds on each other's output: file-checker, scan-or-text, wordbox, where-are-the-regions, what-needs-ocr. Plan and rules: PLAN.md.

## Use

```
cargo build --release --manifest-path repeats/Cargo.toml
wordbox-cli report.pdf | repeats/target/release/repeats-cli -
```

## Results so far

The rules were tuned on the constructed tune split (tools/build_set.py: 240 generated documents, 3,398 pages, seven label styles, covers, chapters, watermarks, duplicate pages) and on 18 real files from govdocs1 003 and 004 that declare /PageLabels, frozen by hash (results/frozen.sha256), then run on held-out data.

Constructed held-out (results/heldout-chains.txt):

| Measure | Result | Target |
|---|---|---|
| Running lines found | 242 of 242 | 95% |
| Lines called running that aren't | 0 of 222 | at most 2% |
| Page labels right | 1,689 of 1,689 | 95% |
| Labels on pages that print none | 0 of 59 | |
| Watermarks found / called where there is none | 26 of 26 / 0 of 94 | |
| Duplicate bodies found / false | 9 of 9 / 0 | |
| Time a page, process start included | 0.34 ms | under 1 ms |

Real files, held out: the 32 files in govdocs1 005, 006 and 007 that declare /PageLabels, 1,473 pages, none looked at before the freeze (results/real-heldout.txt). A printed label was found on 1,200 pages, and on 1,173 of them (97.8%) its number is the one the file declares. This is agreement, not accuracy: files can declare labels they don't print. Most disagreements are in two files.

The first version found printed labels on only 82 of 1,116 pages of the real tune files, because page numbers that alternate sides between odd and even pages ("iii |", "| iv") never form one running line. Labels are now also found by chaining number-like words in the header and footer bands whose values move on with the page, up to three pages apart (results/findings.md).

## Limits

/PageLabels isn't read (it is in the PDF catalogue, not in wordbox's output; the plan is to add it to wordbox). A document with only two labelled pages that change style between them ("i" then "1") gets no labels: there is no step to see. A file that prints Bates numbers ("00001") gets those as its labels, even where it declares others. Labels must be in the top or bottom 12% of the page or the outer 12% at the sides.
