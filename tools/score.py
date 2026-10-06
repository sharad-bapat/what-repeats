"""Score repeats-cli on the constructed set (PLAN.md targets).

For each document: wordbox-cli's JSON (cached in data/constructed/wordbox/), then repeats-cli, then
compared with data/constructed/manifest.json:

  running   a truth running line is found when a predicted running line sits in the same place
            (header, footer, margin) and its pattern contains the truth's (a label printed at the end
            of the header line is part of that line); a predicted running line that matches no truth
            line is false
  labels    per page, the predicted label equals the printed one; a label on a page that prints none
            is false
  marks     a document's watermark found, and no watermark called on a document without one
  dups      pages whose body repeats the page before, found; a duplicate called where there is none

usage: python tools/score.py [--split=tune|heldout] [--misses]
"""
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORDBOX = ROOT.parent / "wordbox" / "extractor" / "target" / "release" / "wordbox-cli.exe"
REPEATS = ROOT / "repeats" / "target" / "release" / "repeats-cli.exe"
DATA = ROOT / "data" / "constructed"


def wordbox_json(item):
    cache = DATA / "wordbox" / (item["id"] + ".json")
    if not cache.exists():
        cache.parent.mkdir(parents=True, exist_ok=True)
        out = subprocess.run([str(WORDBOX), str(DATA / item["file"])], capture_output=True).stdout
        cache.write_bytes(out)
    return cache


def main():
    split = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--split=")), "tune")
    misses = "--misses" in sys.argv
    if split == "heldout":
        sys.path.insert(0, str(Path(__file__).parent))
        from check_frozen import require_frozen
        require_frozen()
    man = json.loads((DATA / "manifest.json").read_text(encoding="utf-8"))
    # wordbox joins words with one space; the truth keeps the spaces the builder drew
    for x in man["items"]:
        for t in x["running"]:
            t["pattern"] = " ".join(t["pattern"].split())
    items = [x for x in man["items"] if x["split"] == split]
    files = [wordbox_json(x) for x in items]
    t0 = time.perf_counter()
    res = subprocess.run([str(REPEATS)] + [str(f) for f in files], capture_output=True, text=True, encoding="utf-8").stdout.splitlines()
    secs = time.perf_counter() - t0
    c = dict.fromkeys(["run_truth", "run_found", "run_pred", "run_false", "lab_truth", "lab_right", "lab_none", "lab_false",
                       "wm_docs", "wm_found", "wm_clean", "wm_false", "dup_truth", "dup_found", "dup_false", "pages"], 0)
    log = []
    for item, line in zip(items, res):
        out = json.loads(line)
        c["pages"] += len(item["pages"])
        pred = out.get("running", [])
        used = set()
        for t in item["running"]:
            c["run_truth"] += 1
            hit = next((p for p in pred if p["where"] == t["where"] and t["pattern"] in p["pattern"]), None)
            if hit:
                c["run_found"] += 1
                used.add(hit["id"])
            else:
                log.append(f"{item['id']} running missed: {t['where']} {t['pattern']!r} ({item['style']})")
        for p in pred:
            c["run_pred"] += 1
            if p["id"] not in used and not any(p["where"] == t["where"] and t["pattern"] in p["pattern"] for t in item["running"]):
                c["run_false"] += 1
                log.append(f"{item['id']} running false: {p['where']} {p['pattern']!r}")
        got = {p["n"]: p["label"] for p in out.get("pages", [])}
        for p in item["pages"]:
            g = got.get(p["n"])
            if p["label"] is None:
                c["lab_none"] += 1
                if g is not None:
                    c["lab_false"] += 1
                    log.append(f"{item['id']} p{p['n']} label false: {g!r}")
            else:
                c["lab_truth"] += 1
                if g == p["label"]:
                    c["lab_right"] += 1
                else:
                    log.append(f"{item['id']} p{p['n']} label {g!r}, printed {p['label']!r} ({item['style']})")
        wms = [w["text"] for w in out.get("watermarks", [])]
        if item["watermark"]:
            c["wm_docs"] += 1
            c["wm_found"] += item["watermark"] in wms
        else:
            c["wm_clean"] += 1
            c["wm_false"] += bool(wms)
        dup = {p["n"]: p["duplicate_of"] for p in out.get("pages", [])}
        for p in item["pages"]:
            if p["duplicate_of"]:
                c["dup_truth"] += 1
                c["dup_found"] += dup.get(p["n"]) == p["duplicate_of"]
            elif dup.get(p["n"]):
                c["dup_false"] += 1
    pct = lambda a, b: f"{a}/{b} ({100 * a / b:.1f}%)" if b else f"{a}/{b}"
    print(f"what-repeats on the constructed {split} split: {len(items)} documents, {c['pages']} pages")
    print(f"  running lines found {pct(c['run_found'], c['run_truth'])} (target 95%); false {pct(c['run_false'], c['run_pred'])} (target at most 2%)")
    print(f"  page labels right {pct(c['lab_right'], c['lab_truth'])} (target 95%); label on a page that prints none {pct(c['lab_false'], c['lab_none'])}")
    print(f"  watermarks found {pct(c['wm_found'], c['wm_docs'])}; called on documents without one {pct(c['wm_false'], c['wm_clean'])}")
    print(f"  duplicate bodies found {pct(c['dup_found'], c['dup_truth'])}; false {c['dup_false']}")
    print(f"  repeats-cli {1000 * secs / max(c['pages'], 1):.3f} ms a page, process start included (target under 1 ms)")
    if misses:
        for m in log[:80]:
            print("   ", m)


if __name__ == "__main__":
    main()
