# what-repeats

What repeats from page to page? Running headers, footers and margin lines, the label each page prints and the sequences those labels form, watermarks, and pages whose body repeats another's.

It reads wordbox's JSON, so it has no PDF parser of its own and runs on any words with boxes, OCR words included. Part of a series that builds on each other's output: file-checker, scan-or-text, wordbox, where-are-the-regions, what-needs-ocr. Plan and rules: PLAN.md.

## Use

```
cargo build --release --manifest-path repeats/Cargo.toml
wordbox-cli report.pdf | repeats/target/release/repeats-cli -
```

## Results so far

The rules were tuned on the constructed tune split (tools/build_set.py: 240 generated documents, 3,398 pages, seven label styles, covers, chapters, watermarks, duplicate pages), frozen by hash (results/frozen.sha256), then run once on the held-out split (results/heldout.txt):

| Held-out, constructed | Result | Target |
|---|---|---|
| Running lines found | 242 of 242 | 95% |
| Lines called running that aren't | 0 of 222 | at most 2% |
| Page labels right | 1,689 of 1,689 | 95% |
| Labels on pages that print none | 0 of 59 | |
| Watermarks found / called where there is none | 26 of 26 / 0 of 94 | |
| Duplicate bodies found / false | 9 of 9 / 0 | |
| Time a page, process start included | 1.3 ms (0.44 ms on tune; held-out ran while another job used the machine) | under 1 ms |

Both splits come from the same generator, so this shows the rules work on the cases it draws, not yet on real reports. Not yet checked: real PDFs (labels against /PageLabels where files state them, and hand-checked reports).

## Limits

/PageLabels isn't read (it is in the PDF catalogue, not in wordbox's output; the plan is to add it to wordbox). A document with only two labelled pages that change style between them ("i" then "1") gets no labels: there is no step to see. Labels must be in the top or bottom 12% of the page or the outer 12% at the sides.
