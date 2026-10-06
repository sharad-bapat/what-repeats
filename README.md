# what-repeats

What repeats from page to page? Running headers and footers, the number each page prints, and the sequences those numbers make, read from the words wordbox gives with their boxes. It also finds watermarks and pages whose body repeats another page's. There's no model and no rendering, and no PDF parser either: the input is wordbox's JSON, so it works on OCR words in the same shape too.

The printed page number matters more than it looks. A contents line that says "4.7 Bit record .... 3-12" means the page that prints "3-12", not the twelfth page of the file, and a reader model that sees one page at a time takes a running header for content.

## Output

One JSON object per file. Boxes are PDF points from the top-left of the page as displayed, as in wordbox. This is a four-page report with a cover, cut to three of its pages:

```json
{"running":[
  {"id":0,"where":"header","text":"Drilling Programme 8/5-12 Section 2","pattern":"Drilling Programme #/#-# Section #","box":[72.0,42.8,219.5,51.8],"pages":[2,3,4],"label":false},
  {"id":1,"where":"footer","text":"Page 2 of 4","pattern":"Page # of #","box":[274.6,784.7,320.7,793.7],"pages":[2,3,4],"label":true}],
 "pages":[
  {"n":1,"label":null,"label_from":null,"sequence":null,"running":[],"duplicate_of":null},
  {"n":2,"label":"2","label_from":1,"sequence":0,"running":[0,1],"duplicate_of":null},
  {"n":4,"label":"4","label_from":1,"sequence":0,"running":[0,1],"duplicate_of":3}],
 "sequences":[{"id":0,"style":"arabic","first_page":2,"last_page":4,"first_label":"2","last_label":"4"}],
 "watermarks":[]}
```

`pattern` is the line with its digits and roman numerals as `#`. `where` is `header`, `footer` or `margin`. `label_from` is the running line the label came from, or `chain` when it came from chaining numbers across pages (below). A sequence's `style` is `arabic`, `roman` or `chapter` (labels like "3-12"). `duplicate_of` is set when a page's body lines are the same as an earlier page's.

## Rules

A line is a wordbox line on the page. Words that come without line numbers, such as the OCR words in what-needs-ocr's merged output, are put into lines first: a word joins a line when it overlaps it in height by half the shorter of the two and sits within two word heights of it across. A line can be running when it sits wholly in the top or bottom 12% of the page, or in the outer 12% at a side. Lines with the same pattern at the same height (within 6 points) form a group, and a group is running when it's on at least two pages and at least a quarter of the document's pages.

The label comes first from a running line whose changing word steps by one from page to page, by its own style: 9 to 10, iv to v, 2-8 to 2-9 or 2-8 to 3-1. Each changing word is tried on its own, since a header can carry a section number as well as the page number. A group qualifies when at least 60% of its consecutive pairs step; ties go to the footer, then to the rightmost word, because page numbers end header lines.

That misses page numbers that change sides between odd and even pages ("iii |" on one page, "| iv" on the next), which never make one running line. So for pages still without a label, every number-like word in the header and footer bands is a candidate, and the longest chain of candidates whose values move on exactly as far as the pages do, up to three pages apart, gives the labels. Chains of three or more pages are kept, longest first, so roman front matter and the arabic pages after it come out as two. Roman numerals stop at c (100), so "mix" and "civ" are words, and a chain doesn't take arabic numbers above 9,999, which are years, depths or codes.

A label that doesn't follow from the one before starts a new sequence. A line set at 2.5 times the median size of the document's lines or more, repeated on two pages or more, is a watermark and isn't counted as running. A page whose body has at least three lines, all the same as an earlier page's, is a duplicate.

## Layout

- `repeats/`: the library and the CLI (`repeats-cli`), Rust, with serde_json as its only dependency.
- `tools/build_set.py`: the constructed set, 240 generated documents. `tools/score.py` scores it, and `tools/real_check.py` compares printed labels with the labels real files declare. `tools/check_frozen.py` checks the frozen source.
- `results/`: the numbers below. `PLAN.md` has the plan and the targets, written before the results.

## Results

The rules were tuned on the constructed tune split and on the 18 files in govdocs1 threads 003 and 004 that declare /PageLabels, frozen by hash (`results/frozen.sha256`), then run on held-out data.

The constructed held-out split (120 documents, 1,748 pages; `results/heldout.txt`), against targets set in the plan:

| Measure | Result | Target |
|---|---|---|
| Running lines found | 242 of 242 | 95% |
| Lines called running that aren't | 0 of 222 | at most 2% |
| Page labels right | 1,689 of 1,689 | 95% |
| Labels on pages that print none | 0 of 59 | |
| Watermarks found, and called where there's none | 26 of 26, 0 of 94 | |
| Duplicate pages found, and false | 9 of 9, 0 | |
| Time a page, process start included | 0.41 ms | under 1 ms |

The constructed documents come from the same generator as the tune split, so they show the rules work on the cases it draws: seven label styles, covers, chapters, a section number in the header, margin lines, watermarks and repeated pages. Real files are the harder test. The 32 files in govdocs1 threads 005, 006 and 007 that declare /PageLabels, 1,473 pages, weren't looked at before the freeze (`results/real-heldout.txt`). A printed label was found on 1,200 of those pages, and on 1,173 of them (97.8%) its number is the one the file declares. That's agreement, not accuracy: a file can declare labels it doesn't print. Four files hold 26 of the 27 disagreements.

The first version found printed labels on only 82 of the 1,116 pages in the real tune files, because of the odd and even sides. With the chains it finds them on 1,022, and 1,018 have the declared number (`results/real-tune.txt`, `results/findings.md`).

Changes made after the held-out run are measured the same way in `results/after-heldout.md`.

## Limits

/PageLabels isn't read: it's in the PDF's catalogue, not in wordbox's output. A document with only two labelled pages that change style between them ("i", then "1") gets no labels, because there's no step to see. A file that prints Bates numbers ("00001") gets those as its labels even where it declares others. Labels have to sit in the bands above; a page number printed in the middle of the page isn't found.

## Commands

```
cargo build --release --manifest-path repeats/Cargo.toml
wordbox-cli report.pdf | repeats/target/release/repeats-cli -
repeats/target/release/repeats-cli report.wordbox.json
python tools/build_set.py
python tools/score.py [--split=heldout] [--misses]
python tools/real_check.py [--heldout] [--list]
python tools/check_frozen.py
```

wordbox-cli comes from [wordbox](https://github.com/sharad-bapat/wordbox); the scorers expect it built in a `wordbox` folder next to this one.

## Licence

MIT. See [LICENSE](LICENSE).
