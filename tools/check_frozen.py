"""Check the frozen source: every file in results/frozen.sha256 must hash as recorded, and the
release binary must be newer than all of them. tools/score.py refuses the held-out split unless
this passes.

What is frozen: the crate (Cargo.toml, Cargo.lock, every file in repeats/src), the set builder,
the scorer, the constructed set's manifest (each document pinned by sha256) and this file.
Line endings are normalised to LF first, so a checkout that converts them doesn't count as a change.

usage: python tools/check_frozen.py            check
       python tools/check_frozen.py --write    record the current source (at a freeze only)
"""
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECORD = ROOT / "results" / "frozen.sha256"
CLI = ROOT / "repeats" / "target" / "release" / "repeats-cli.exe"


def names():
    src = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / "repeats" / "src").glob("*.rs"))
    return ["repeats/Cargo.toml", "repeats/Cargo.lock"] + src + ["tools/build_set.py", "tools/score.py", "tools/check_frozen.py",
                                                                  "data/constructed/manifest.json"]


def digest(name):
    return hashlib.sha256((ROOT / name).read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def require_frozen():
    if not RECORD.exists():
        raise SystemExit("not frozen yet: results/frozen.sha256 is missing")
    rec = dict(line.split("  ", 1)[::-1] for line in RECORD.read_text(encoding="utf-8").splitlines() if line.strip())
    bad = [n for n in names() if rec.get(n) != digest(n)] + [n for n in rec if n not in names()]
    if bad:
        raise SystemExit(f"frozen source changed: {', '.join(bad)}")
    newest = max((ROOT / n).stat().st_mtime for n in names() if n.startswith("repeats/"))
    if not CLI.exists() or CLI.stat().st_mtime < newest:
        raise SystemExit("rebuild repeats-cli: the release binary is older than the frozen source")
    print("frozen source: ok")


if __name__ == "__main__":
    if "--write" in sys.argv:
        RECORD.parent.mkdir(parents=True, exist_ok=True)
        RECORD.write_text("".join(f"{digest(n)}  {n}\n" for n in names()), encoding="utf-8", newline="\n")
        print(f"recorded {len(names())} files")
    else:
        require_frozen()
