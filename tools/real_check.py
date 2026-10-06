"""Printed labels against the labels real files declare (/PageLabels), on govdocs1 003 and 004.

Every PDF in those threads with three or more pages whose /PageLabels differ from plain 1, 2, 3 is
read by wordbox-cli and repeats-cli, and each page's printed label is compared with the declared
one (read with PyMuPDF). This is agreement, not accuracy: a file can declare labels it doesn't
print, or print labels it doesn't declare. Two figures: exact agreement, and agreement on the
number once a declared prefix such as "Black:" or "A-" is set aside.

usage: python tools/real_check.py [--list]
"""
import json
import re
import subprocess
import sys
from pathlib import Path

import fitz

ROOT = Path(__file__).resolve().parent.parent
GOVDOCS = Path("C:/Users/sharad/Downloads/05_Research-Reference/Datasets/govdocs1")
WORDBOX = ROOT.parent / "wordbox" / "extractor" / "target" / "release" / "wordbox-cli.exe"
REPEATS = ROOT / "repeats" / "target" / "release" / "repeats-cli.exe"


def tail(label):
    m = re.search(r"([0-9]+|[ivxlcdmIVXLCDM]+)$", label or "")
    return m.group(1).lower() if m else None


def main():
    show = "--list" in sys.argv
    files = []
    for f in sorted(list((GOVDOCS / "003").glob("*.pdf")) + list((GOVDOCS / "004").glob("*.pdf"))):
        try:
            with fitz.open(f) as d:
                if d.is_encrypted or d.page_count < 3:
                    continue
                labs = [d[i].get_label() for i in range(d.page_count)]
        except Exception:
            continue
        if any(labs) and labs != [str(i + 1) for i in range(len(labs))]:
            files.append((f, labs))
    tot = dict(pages=0, declared=0, printed=0, exact=0, number=0)
    for f, labs in files:
        wb = subprocess.run([str(WORDBOX), str(f)], capture_output=True).stdout
        out = json.loads(subprocess.run([str(REPEATS), "-"], input=wb, capture_output=True).stdout.decode("utf-8").splitlines()[0])
        got = {p["n"]: p["label"] for p in out.get("pages", [])}
        row = dict(pages=len(labs), declared=sum(1 for x in labs if x), printed=0, exact=0, number=0)
        for i, lab in enumerate(labs):
            g = got.get(i + 1)
            if g is None:
                continue
            row["printed"] += 1
            row["exact"] += g == lab
            row["number"] += tail(g) is not None and tail(g) == tail(lab)
        for k in tot:
            tot[k] += row[k]
        if show:
            first = next((i for i in range(len(labs)) if got.get(i + 1)), None)
            print(f"  {f.parent.name}/{f.name}: {row['pages']} pages, printed label on {row['printed']}, exact {row['exact']}, number {row['number']}"
                  + (f"; first: printed {got.get(first + 1)!r} declared {labs[first]!r}" if first is not None else ""))
    print(f"{len(files)} files declaring /PageLabels, {tot['pages']} pages")
    print(f"  a printed label found on {tot['printed']} pages; of those, the same as declared {tot['exact']}, the same number {tot['number']}")


if __name__ == "__main__":
    main()
