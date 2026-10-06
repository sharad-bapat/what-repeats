"""Build the constructed set: multi-page PDFs whose running lines, page labels, watermarks and
duplicate pages are known because this script drew them (PLAN.md, "Ground truth and tests").

Each document draws, by choices made from its own seeded generator:

  header     a running line in the top band: the document's title, sometimes with a section number
             that changes by chapter (digits masked, it is still one running line)
  label      the printed page label, in one of seven styles: "3", "Page 3", "Page 3 of 20", "- 3 -",
             chapter-page "2-5", roman front matter then arabic from 1, and "3" in the header's right end;
             placed centre or right in the footer band, or in the header
  cover      sometimes a first page with no header and no label; numbering then starts on it or after it
  margin     sometimes a line up the left margin ("Company Confidential"), on every page
  watermark  sometimes a large light-grey word ("DRAFT"), turned 45 degrees, on every page
  duplicate  sometimes one page drawn twice, byte for byte the same words
  decoys     body text with numbers that look like labels: numbered headings, table rows of numbers,
             and a line at the foot of the body on some pages that isn't running
  /PageLabels sometimes, stating the same labels as the printed ones (reportlab addPageLabel)

Truth per document in data/constructed/manifest.json: every running line (its text with digits as
#, where it sits, the pages it's on), every page's printed label or null, the watermark, duplicate
pages and the /PageLabels written. The split (tune or heldout) is by a hash of the document id. The
output is deterministic: the same seed writes the same files, and the manifest pins each by sha256.

usage: python tools/build_set.py [--count N]
"""
import hashlib
import json
import random
import re
import sys
from pathlib import Path

from reportlab.lib.pagesizes import A4, letter
from reportlab.pdfgen import canvas


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "data" / "constructed"
SEED = 20261006
STYLES = ["arabic", "page", "page_of", "dash", "chapter", "roman_arabic", "header_right"]
WORDS = ("well report drilling completion casing cement mud log depth formation sand shale core sample test pressure "
         "temperature summary geology reservoir operations safety equipment rig crew water gas oil interval section "
         "results analysis program casing liner string fluid density weight survey deviation azimuth bottom hole").split()
TITLES = ["Final Well Report", "Completion Report", "Annual Review", "Drilling Programme", "Geological Summary",
          "Operations Manual", "Technical Note", "Field Development Plan"]
ROMAN = [(1000, "m"), (900, "cm"), (500, "d"), (400, "cd"), (100, "c"), (90, "xc"), (50, "l"), (40, "xl"), (10, "x"), (9, "ix"), (5, "v"), (4, "iv"), (1, "i")]


def roman(n):
    out = ""
    for v, s in ROMAN:
        while n >= v:
            out, n = out + s, n - v
    return out


def mask(text):
    return re.sub(r"\d+", "#", text)


def body_lines(rng, n):
    lines = []
    for _ in range(n):
        r = rng.random()
        if r < 0.12:
            lines.append(f"{rng.randint(1, 12)}.{rng.randint(1, 9)} {' '.join(rng.choice(WORDS) for _ in range(rng.randint(1, 4))).title()}")
        elif r < 0.25:
            lines.append("    ".join(str(rng.randint(1, 4000)) for _ in range(rng.randint(3, 6))))
        else:
            lines.append(" ".join(rng.choice(WORDS) for _ in range(rng.randint(6, 14))).capitalize() + ".")
    return lines


def plan_doc(i):
    rng = random.Random(SEED * 1000 + i)
    n = rng.randint(3, 24)
    d = {"id": f"r{i:04d}", "pages": n, "size": rng.choice(["A4", "letter"]), "style": rng.choice(STYLES),
         "title": f"{rng.choice(TITLES)} {rng.randint(1, 35)}/{rng.randint(1, 12)}-{rng.randint(1, 20)}",
         "header": rng.random() < 0.8, "section_in_header": rng.random() < 0.3, "cover": rng.random() < 0.5,
         "cover_counts": rng.random() < 0.5, "place": rng.choice(["centre", "right"]), "margin": rng.random() < 0.15,
         "watermark": rng.choice(["DRAFT", "CONFIDENTIAL"]) if rng.random() < 0.2 else None,
         "page_labels": rng.random() < 0.5, "duplicate": rng.randint(2, n) if n >= 4 and rng.random() < 0.15 else None,
         "front": rng.randint(2, 4), "chapters": sorted(rng.sample(range(2, n + 1), k=min(n - 1, rng.randint(1, 3))))}
    if d["style"] == "header_right":
        d["header"] = True
    # a cover isn't a body page to copy: it also draws the title, so page 2 would never match it
    if d["cover"] and d["duplicate"] == 2:
        d["duplicate"] = None
    d["split"] = "tune" if int(hashlib.sha256(d["id"].encode()).hexdigest(), 16) % 2 == 0 else "heldout"
    return d, rng


def labels_of(d):
    """The printed label of each page (None on a cover without one), and the /PageLabels ranges."""
    n, style = d["pages"], d["style"]
    first = 2 if d["cover"] else 1           # first page that prints a label
    start = first if d["cover_counts"] or not d["cover"] else 1   # the number it prints
    out, ranges = [None] * (n + 1), []
    if style == "roman_arabic":
        front = min(d["front"], n - first)   # front matter in roman, then arabic from 1
        for k, p in enumerate(range(first, first + front)):
            out[p] = roman(k + 1)
        for k, p in enumerate(range(first + front, n + 1)):
            out[p] = str(k + 1)
        ranges = [(first - 1, "r", 1, ""), (first + front - 1, "D", 1, "")]
    elif style == "chapter":
        ch, k = 1, 0
        bounds = set(c for c in d["chapters"] if c > first)
        for p in range(first, n + 1):
            if p in bounds:
                ch, k = ch + 1, 0
            k += 1
            out[p] = f"{ch}-{k}"
            if k == 1:
                ranges.append((p - 1, "D", 1, f"{ch}-"))
    else:
        for p in range(first, n + 1):
            out[p] = str(start + p - first)
        ranges = [(first - 1, "D", start, "")]
    if d["cover"]:
        ranges = [(0, None, None, "Cover")] + ranges
    return out, ranges


def printed(d, label, n_labels):
    s = d["style"]
    if s == "page":
        return f"Page {label}"
    if s == "page_of":
        return f"Page {label} of {n_labels}"
    if s == "dash":
        return f"- {label} -"
    return label


def build(i):
    d, rng = plan_doc(i)
    w, h = A4 if d["size"] == "A4" else letter
    labels, ranges = labels_of(d)
    # "Page 3 of N" counts to the last page number printed, as a real document does
    n_labels = max((int(x) for x in labels if x and x.isdigit()), default=0)
    path = OUT / d["split"] / f"{d['id']}.pdf"
    path.parent.mkdir(parents=True, exist_ok=True)
    c = canvas.Canvas(str(path), pagesize=(w, h), invariant=1)
    running = {}

    def run(key, where, text, page):
        e = running.setdefault((key, where), {"where": where, "pattern": mask(text), "pages": []})
        e["pages"].append(page)

    chapter, bodies = 1, {}
    for p in range(1, d["pages"] + 1):
        if p in d["chapters"]:
            chapter += 1
        cover = d["cover"] and p == 1
        src = d["duplicate"] - 1 if d["duplicate"] and p == d["duplicate"] else p
        if src not in bodies:
            bodies[src] = body_lines(random.Random(SEED + i * 100 + src), rng.randint(12, 32) if not cover else 3)
        lines = bodies[src]
        c.setFont("Helvetica", 10)
        y = h - 110
        for line in lines:
            c.drawString(72, y, line[:95])
            y -= 15
            if y < 110:
                break
        if cover:
            c.setFont("Helvetica-Bold", 20)
            c.drawCentredString(w / 2, h / 2, d["title"])
        if d["header"] and not cover:
            c.setFont("Helvetica", 9)
            text = d["title"] + (f"  Section {chapter}" if d["section_in_header"] else "")
            c.drawString(72, h - 50, text)
            run("title", "header", text, p)
        if labels[p]:
            t = printed(d, labels[p], n_labels)
            c.setFont("Helvetica", 9)
            if d["style"] == "header_right":
                c.drawRightString(w - 72, h - 50, t)
            elif d["place"] == "centre":
                c.drawCentredString(w / 2, 50, t)
            else:
                c.drawRightString(w - 72, 50, t)
            where = "header" if d["style"] == "header_right" else "footer"
            run("label", where, t if d["style"] not in ("roman_arabic", "chapter") else t, p)
        if d["margin"]:
            c.saveState()
            c.translate(40, h / 2)
            c.rotate(90)
            c.setFont("Helvetica", 8)
            c.drawCentredString(0, 0, "Company Confidential")
            c.restoreState()
            run("margin", "margin", "Company Confidential", p)
        if d["watermark"]:
            c.saveState()
            c.setFillGray(0.85)
            c.translate(w / 2, h / 2)
            c.rotate(45)
            c.setFont("Helvetica-Bold", 72)
            c.drawCentredString(0, 0, d["watermark"])
            c.restoreState()
        c.showPage()
    if d["page_labels"]:
        for start, style, first, prefix in ranges:
            kw = {"prefix": prefix} if prefix else {}
            if style:
                c.addPageLabel(start, style={"D": "ARABIC", "r": "ROMAN_LOWER"}[style], start=first, **kw)
            else:
                c.addPageLabel(start, **kw)
    c.save()
    pages = [{"n": p, "label": labels[p], "duplicate_of": (d["duplicate"] - 1 if d["duplicate"] == p else None)} for p in range(1, d["pages"] + 1)]
    run_list = []
    for (key, where), e in running.items():
        if key == "label" and d["style"] in ("roman_arabic", "chapter"):
            e["pattern"] = "#" if d["style"] == "roman_arabic" else "#-#"
        if len(e["pages"]) >= 2:
            run_list.append({"id": len(run_list), "kind": key, **e})
    return {"id": d["id"], "file": f"{d['split']}/{d['id']}.pdf", "split": d["split"], "pages": pages, "running": run_list,
            "watermark": d["watermark"], "page_labels": [list(r) for r in ranges] if d["page_labels"] else None,
            "style": d["style"], "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def main():
    count = int(sys.argv[sys.argv.index("--count") + 1]) if "--count" in sys.argv else 240
    items = [build(i) for i in range(count)]
    man = {"seed": SEED, "count": count, "items": items}
    (OUT / "manifest.json").write_text(json.dumps(man, indent=1), encoding="utf-8")
    split = {s: sum(1 for x in items if x["split"] == s) for s in ("tune", "heldout")}
    print(f"{count} documents, {sum(len(x['pages']) for x in items)} pages: {split}")


if __name__ == "__main__":
    main()
