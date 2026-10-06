# Findings

## PyMuPDF's Page.get_label returns a UTF-16 prefix as raw hex (6 October 2026)

govdocs1 003837 declares its page labels with a prefix written as a hex string: the label dictionary (object 241) is `<< /St 69 /P <FEFF0053006500630031003A> /S /D >>`, which is "Sec1:" in UTF-16BE with a byte-order mark. PyMuPDF 1.24.9 (`fitz.VersionBind`) decodes the key correctly when it is read directly, `doc.xref_get_key(241, "P")` giving `('string', 'Sec1:')`, but `doc[0].get_label()` returns `<FEFF0053006500630031003A>69`, the prefix's hex digits as they stand in the file. 003405 shows the same. Checked with PyMuPDF 1.24.9 only; newer releases not checked.

Effect here: tools/real_check.py compares printed labels with declared ones through get_label, so on 003837 the exact comparison fails though the numbers agree (9 of 9 pages); the "same number" figure sets prefixes aside and is not affected.

## Printed labels against /PageLabels on real files (6 October 2026)

tools/real_check.py, results/real-check.txt: 18 files in govdocs1 003 and 004 declare /PageLabels other than plain 1, 2, 3 (1,116 pages). what-repeats found a printed label on 82 pages; 69 equal the declared label and 78 have the same number. So the labels it finds are nearly always right, but it finds few: 11 files get none. The main cause, seen on 003737 and 003399, is page numbers that alternate sides between odd and even pages ("iii |", "| iv", "v |"; a journal footer on one side of alternate pages), which split into two running groups that never step between adjacent pages. The constructed set has no such documents. These 18 files are now tune data for the fix; it will be tested on files not looked at.
