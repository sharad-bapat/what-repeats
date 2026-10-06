# Findings

## Page numbers that change sides (6 October 2026)

The first version took labels only from running lines: a line with the same pattern at the same height on many pages, whose changing word goes up by one. On the 18 files in govdocs1 003 and 004 that declare /PageLabels (1,116 pages), it found a printed label on 82 pages, 78 of them with the declared number (results/real-tune-first.txt). Right when it answered, but it hardly answered: 11 files got no labels at all.

The files that got none mostly print the number on the outer edge, so it changes sides between odd and even pages. 003737 has "iii |" at the bottom left of one page and "| iv" at the bottom right of the next. 003399, a journal, puts "Vol. 9(4), 2007 487" in the footer of alternate pages only. Each side forms its own running line, present on every other page, and a line that's only on pages 1, 3 and 5 never steps between neighbouring pages. The constructed set has no such documents, which is why it scored 100%.

The fix chains number-like words instead: for a page still without a label, every number in its header and footer bands is a candidate, and a chain links candidates whose values move on exactly as far as the pages do, up to three pages apart. The longest chains win. With that (frozen at 11a1f70) the same 18 files have printed labels on 1,022 pages, 1,018 with the declared number (results/real-tune.txt). On the 32 files in govdocs1 005 to 007 that declare /PageLabels, not looked at before the freeze, it finds labels on 1,200 of 1,473 pages, and 1,173 of them have the declared number (results/real-heldout.txt). The constructed held-out split stayed at 100% on every measure.

## PyMuPDF's Page.get_label leaves a UTF-16 prefix undecoded (6 October 2026)

govdocs1 003837 writes its page-label prefix as a hex string. The label dictionary, object 241, is `<< /St 69 /P <FEFF0053006500630031003A> /S /D >>`, and the prefix is "Sec1:" in UTF-16BE with a byte-order mark. PyMuPDF 1.24.9 decodes the key when it's read directly, `doc.xref_get_key(241, "P")` giving `('string', 'Sec1:')`, but `doc[0].get_label()` returns `<FEFF0053006500630031003A>69`, the hex digits as they stand in the file. 003405 does the same. I've only checked 1.24.9.

tools/real_check.py reads declared labels through get_label, so on these files the exact comparison fails though the numbers agree. That's why the results give a second figure, agreement on the number with any prefix set aside, and why the exact figure is lower.
