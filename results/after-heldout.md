# Changes after the held-out run

Each change here came after the held-out run, and is measured the same way and reported apart from it.

## Words without line numbers (6 October 2026)

what-repeats read lines from wordbox's line numbers, so words that come without them were all taken as one line per page. OCR words have none: what-needs-ocr's merged output gives each word its text, box, source and confidence. So a page of OCR words was read as one long line, and nothing on it could repeat.

A page whose words carry no line numbers now gets them from `group_lines`. Words are taken top to bottom by their middles, and a word joins the first line it overlaps in height by at least half the shorter of the two (`LINE_OVERLAP`), if it sits no more than two word heights from that line across (`LINE_GAP`). Otherwise it starts a line. The words of each line are read left to right. A page whose words have line numbers is read as before.

The constructed sets and the real files all come from wordbox, which numbers its lines, so their results can't move, and they don't: the tune and held-out splits and both real-file checks give the same numbers as `results/tune.txt`, `results/heldout.txt`, `results/real-tune.txt` and `results/real-heldout.txt`. A test checks that a document with its line numbers taken away and its words reversed gives the same output as the original.

On a 12-page scanned completion report from the Norwegian Offshore Directorate, read with Tesseract through what-needs-ocr, the old code found no running line and no label. Now it finds "CONFIDENTIAL" as a running header on pages 3 to 7, and the printed labels 2, 4, 5 and 6 on pages 4, 6, 7 and 8. It also shows a second problem. "CONFIDENTIAL" is printed on pages 2 and 8 too, but the scan sits 6 to 7 points higher there, past the 6 points that make two lines "the same height". That needs its own rule, tuned on scanned pages.

## Each page's box for a running line (6 October 2026)

Each running line now also has `boxes`, its box on every page it's found on, in the order of `pages`. `box` gave only the first page's, and on scans a header sits a little higher or lower from page to page, so a caller that wants to drop running lines from each page's text had no exact box for the other pages. The field is added, nothing else changes, and the tune and held-out results are the same as in `results/tune.txt` and `results/heldout.txt`.
