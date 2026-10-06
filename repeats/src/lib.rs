//! What repeats from page to page (PLAN.md): running headers, footers and margin lines, printed page
//! labels and their sequences, watermarks and duplicate pages, from wordbox's JSON for a file. No PDF
//! parser here: the words, their boxes, lines and sizes are wordbox's (D4, the tools compose). Fixed
//! rules, the constants below; the same input always gives the same output.

use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The header band is the top BAND of the page, the footer band the bottom BAND, the margins the
/// outer MARGIN on each side; a line wholly inside one of them may be running.
pub const BAND: f64 = 0.12;
pub const MARGIN: f64 = 0.12;
/// A line is running when the same masked text sits at the same height (within Y_TOL points) on at
/// least RUN_PAGES pages and at least RUN_SHARE of the document's pages.
pub const RUN_PAGES: usize = 2;
pub const RUN_SHARE: f64 = 0.25;
pub const Y_TOL: f64 = 6.0;
/// A running group is the page-label line when its changing words step by one from page to page on
/// at least LABEL_STEPS of the consecutive pairs it has.
pub const LABEL_STEPS: f64 = 0.6;
/// Label chains: a number-like word in the header or footer band continues a chain when its value
/// has moved on exactly as many steps as the page has, at most CHAIN_GAP pages on (page numbers that
/// alternate sides skip a page in each place). A chain needs CHAIN_MIN pages.
pub const CHAIN_GAP: i64 = 3;
pub const CHAIN_MIN: usize = 3;
/// Arabic labels above this are years, codes or depths, not page numbers.
pub const ARABIC_MAX: i64 = 9999;

/// A line set at least WATERMARK times the document's median word size, on RUN_PAGES pages or more,
/// is a watermark.
pub const WATERMARK: f64 = 2.5;
/// A page whose body holds at least DUP_LINES lines, all the same as an earlier page's, is a duplicate.
pub const DUP_LINES: usize = 3;

#[derive(Clone, Debug)]
pub struct Line { pub page: i64, pub text: String, pub key: String, pub band: &'static str, pub b: [f64; 4], pub size: f64 }

fn f(v: &Value, k: &str) -> f64 { v.get(k).and_then(Value::as_f64).unwrap_or(0.0) }

/// Roman page labels go no higher than this (front matter is short).
pub const ROMAN_MAX: i64 = 100;

/// A word that is a roman numeral from i to ROMAN_MAX, case-insensitive; its value.
pub fn roman(s: &str) -> Option<i64> {
    let t = s.to_ascii_lowercase();
    if t.is_empty() || !t.chars().all(|c| "ivxlcdm".contains(c)) { return None; }
    let val = |c| match c { 'i' => 1, 'v' => 5, 'x' => 10, 'l' => 50, 'c' => 100, 'd' => 500, _ => 1000 };
    let cs: Vec<i64> = t.chars().map(val).collect();
    let mut n = 0;
    for i in 0..cs.len() { if i + 1 < cs.len() && cs[i] < cs[i + 1] { n -= cs[i] } else { n += cs[i] } }
    // only the canonical spelling up to ROMAN_MAX counts, so "dim" and "mix" (1009) stay words
    (n <= ROMAN_MAX && to_roman(n) == t).then_some(n)
}

fn to_roman(mut n: i64) -> String {
    let mut s = String::new();
    for (v, r) in [(1000, "m"), (900, "cm"), (500, "d"), (400, "cd"), (100, "c"), (90, "xc"), (50, "l"), (40, "xl"), (10, "x"), (9, "ix"), (5, "v"), (4, "iv"), (1, "i")] {
        while n >= v { s.push_str(r); n -= v; }
    }
    s
}

/// A line's text with each run of digits, and each word that is a roman numeral, as `#`.
pub fn mask(text: &str) -> String {
    text.split_whitespace().map(|w| {
        if roman(w).is_some() { return "#".to_string(); }
        let mut out = String::new();
        let mut in_digits = false;
        for c in w.chars() {
            if c.is_ascii_digit() { if !in_digits { out.push('#'); } in_digits = true; } else { out.push(c); in_digits = false; }
        }
        out
    }).collect::<Vec<_>>().join(" ")
}

/// A label's value for stepping: arabic n, roman n, or chapter-page (c, k).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Val { Arabic(i64), Roman(i64), Chapter(i64, i64) }

pub fn value(label: &str) -> Option<Val> {
    if let Ok(n) = label.parse::<i64>() { return Some(Val::Arabic(n)); }
    if let Some(n) = roman(label) { return Some(Val::Roman(n)); }
    let (a, b) = label.split_once('-')?;
    Some(Val::Chapter(a.parse().ok()?, b.parse().ok()?))
}

/// The label after `a` by one step, by its own style.
fn steps(a: Val, b: Val) -> bool {
    match (a, b) {
        (Val::Arabic(x), Val::Arabic(y)) | (Val::Roman(x), Val::Roman(y)) => y == x + 1,
        (Val::Chapter(c, k), Val::Chapter(d, j)) => (d == c && j == k + 1) || (d == c + 1 && j == 1),
        _ => false,
    }
}

/// `b` on page `q` is `a` on page `p` moved on by the page difference, in the same style.
fn fits(a: Val, p: i64, b: Val, q: i64) -> bool {
    let d = q - p;
    if d <= 0 { return false; }
    match (a, b) {
        (Val::Arabic(x), Val::Arabic(y)) | (Val::Roman(x), Val::Roman(y)) => y - x == d,
        (Val::Chapter(c, k), Val::Chapter(e, j)) => (e == c && j - k == d) || (e == c + 1 && j >= 1 && j <= d),
        _ => false,
    }
}

/// Labels by chains (CHAIN_GAP, CHAIN_MIN) over the number-like words of the header and footer bands:
/// the longest chain first, then the longest among what is left, so roman front matter and the arabic
/// pages after it come out as two. Ties go to the footer, then to the rightmost word.
pub fn chain_labels(ls: &[Line]) -> BTreeMap<i64, String> {
    let mut cand: Vec<(i64, String, Val, u8, f64)> = Vec::new(); // page, word, value, band rank, x
    for l in ls.iter().filter(|l| l.band == "header" || l.band == "footer") {
        let words: Vec<&str> = l.text.split_whitespace().collect();
        let n = words.len().max(1) as f64;
        for (k, w) in words.iter().enumerate() {
            let t = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').trim_matches('-');
            let v = match value(t) { Some(Val::Arabic(x)) if x > ARABIC_MAX || x < 0 => None, v => v };
            if let Some(v) = v {
                let x = l.b[0] + (l.b[2] - l.b[0]) * (k as f64 + 0.5) / n;
                cand.push((l.page, t.to_string(), v, if l.band == "footer" { 2 } else { 1 }, x));
            }
        }
    }
    cand.sort_by(|a, b| a.0.cmp(&b.0).then(a.4.total_cmp(&b.4)));
    let mut out = BTreeMap::new();
    let mut live = vec![true; cand.len()];
    loop {
        // len, band score, x score, previous
        let mut best: Vec<(usize, u32, f64, Option<usize>)> = vec![(0, 0, 0.0, None); cand.len()];
        for i in 0..cand.len() {
            if !live[i] { continue; }
            best[i] = (1, cand[i].3 as u32, cand[i].4, None);
            for j in (0..i).rev() {
                if cand[i].0 - cand[j].0 > CHAIN_GAP { break; }
                if !live[j] || !fits(cand[j].2, cand[j].0, cand[i].2, cand[i].0) { continue; }
                let c = (best[j].0 + 1, best[j].1 + cand[i].3 as u32, best[j].2 + cand[i].4, Some(j));
                if (c.0, c.1, c.2).partial_cmp(&(best[i].0, best[i].1, best[i].2)) == Some(std::cmp::Ordering::Greater) { best[i] = c; }
            }
        }
        let end = (0..cand.len()).filter(|&i| live[i]).max_by(|&a, &b| (best[a].0, best[a].1, best[a].2).partial_cmp(&(best[b].0, best[b].1, best[b].2)).unwrap());
        let Some(mut i) = end else { break };
        if best[i].0 < CHAIN_MIN { break; }
        let mut pages = Vec::new();
        loop {
            out.insert(cand[i].0, cand[i].1.clone());
            pages.push(cand[i].0);
            match best[i].3 { Some(j) => i = j, None => break }
        }
        for (k, c) in cand.iter().enumerate() { if pages.contains(&c.0) { live[k] = false; } }
    }
    out
}

/// Lines per page from wordbox's words (its line numbers), with their bands.
pub fn lines(doc: &Value) -> Vec<Line> {
    let mut out = Vec::new();
    for p in doc.get("pages").and_then(Value::as_array).into_iter().flatten() {
        let (n, w, h) = (p.get("n").and_then(Value::as_i64).unwrap_or(0), f(p, "width"), f(p, "height"));
        let mut by_line: BTreeMap<i64, Vec<&Value>> = BTreeMap::new();
        for wd in p.get("words").and_then(Value::as_array).into_iter().flatten() {
            if wd.get("offpage").is_some() || wd.get("invisible").is_some() { continue; }
            by_line.entry(wd.get("line").and_then(Value::as_i64).unwrap_or(-1)).or_default().push(wd);
        }
        for ws in by_line.values() {
            let text = ws.iter().filter_map(|w| w.get("t").and_then(Value::as_str)).collect::<Vec<_>>().join(" ");
            if text.trim().is_empty() { continue; }
            let b = ws.iter().fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |b, w| [b[0].min(f(w, "x0")), b[1].min(f(w, "y0")), b[2].max(f(w, "x1")), b[3].max(f(w, "y1"))]);
            let size = ws.iter().map(|w| f(w, "size")).fold(0.0, f64::max);
            let band = if b[3] <= h * BAND { "header" } else if b[1] >= h * (1.0 - BAND) { "footer" }
                       else if b[2] <= w * MARGIN || b[0] >= w * (1.0 - MARGIN) { "margin" } else { "body" };
            out.push(Line { page: n, key: mask(&text), text, band, b, size });
        }
    }
    out
}

/// The whole analysis of one wordbox document, as the output JSON.
pub fn analyse(doc: &Value) -> Value {
    let pages: Vec<i64> = doc.get("pages").and_then(Value::as_array).into_iter().flatten().filter_map(|p| p.get("n").and_then(Value::as_i64)).collect();
    let ls = lines(doc);
    let mut sizes: Vec<f64> = ls.iter().map(|l| l.size).filter(|&s| s > 0.0).collect();
    sizes.sort_by(f64::total_cmp);
    let median = if sizes.is_empty() { 0.0 } else { sizes[sizes.len() / 2] };
    let need = RUN_PAGES.max((RUN_SHARE * pages.len() as f64).ceil() as usize);

    // watermarks: large type repeated on many pages, wherever it sits
    let mut marks: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for l in &ls { if median > 0.0 && l.size >= WATERMARK * median { marks.entry(l.text.clone()).or_default().push(l.page); } }
    marks.retain(|_, v| { v.dedup(); v.len() >= RUN_PAGES });

    // running groups: same band, same masked text, same height
    let mut groups: Vec<(String, &'static str, f64, Vec<usize>)> = Vec::new(); // key, band, y, line indexes
    for (i, l) in ls.iter().enumerate() {
        if l.band == "body" || marks.contains_key(&l.text) { continue; }
        let y = (l.b[1] + l.b[3]) / 2.0;
        match groups.iter_mut().find(|g| g.0 == l.key && g.1 == l.band && (g.2 - y).abs() <= Y_TOL) {
            Some(g) => g.3.push(i),
            None => groups.push((l.key.clone(), l.band, y, vec![i])),
        }
    }
    let mut running = Vec::new(); // (key, band, line indexes, one per page)
    for (key, band, _, idx) in groups {
        let mut per_page: BTreeMap<i64, usize> = BTreeMap::new();
        for i in idx { per_page.entry(ls[i].page).or_insert(i); }
        if per_page.len() >= need { running.push((key, band, per_page.into_values().collect::<Vec<usize>>())); }
    }

    // the label group: changing words that step by one from page to page
    // ties go to the footer over the header, then to the rightmost word (a page number ends a header
    // line): a short document that starts a section on every page steps both
    let mut best: Option<(usize, (f64, u8, usize), BTreeMap<i64, String>)> = None;
    for (gi, (_, band, idx)) in running.iter().enumerate() {
        if *band == "margin" { continue; }
        let toks: Vec<Vec<&str>> = idx.iter().map(|&i| ls[i].text.split_whitespace().collect()).collect();
        let len = toks[0].len();
        if toks.iter().any(|t| t.len() != len) { continue; }
        let varying: Vec<usize> = (0..len).filter(|&k| toks.iter().any(|t| t[k] != toks[0][k])).collect();
        // each changing word on its own is a candidate label (a header can carry a section number and
        // the page number), and all of them together (a label printed as two words)
        let mut cands: Vec<Vec<usize>> = varying.iter().map(|&k| vec![k]).collect();
        if varying.len() > 1 { cands.push(varying.clone()); }
        for cand in cands {
            let labels: BTreeMap<i64, String> = idx.iter().zip(&toks).map(|(&i, t)| (ls[i].page, cand.iter().map(|&k| t[k]).collect::<Vec<_>>().join(" "))).collect();
            let seq: Vec<(i64, Option<Val>)> = labels.iter().map(|(p, s)| (*p, value(s))).collect();
            let pairs = seq.windows(2).filter(|w| w[1].0 == w[0].0 + 1).count();
            let good = seq.windows(2).filter(|w| w[1].0 == w[0].0 + 1 && matches!((w[0].1, w[1].1), (Some(a), Some(b)) if steps(a, b))).count();
            let score = if pairs == 0 { 0.0 } else { good as f64 / pairs as f64 };
            let rank = (score, if *band == "footer" { 2 } else { 1 }, *cand.last().unwrap());
            if score >= LABEL_STEPS && best.as_ref().map_or(true, |b| rank.partial_cmp(&b.1) == Some(std::cmp::Ordering::Greater)) { best = Some((gi, rank, labels)); }
        }
    }

    // labels: the running line's where one steps from page to page, and chains of number-like words
    // in the bands for the pages it leaves (numbers that alternate sides, labels on only some pages)
    let mut label_of: BTreeMap<i64, String> = best.as_ref().map(|b| b.2.clone()).unwrap_or_default();
    for (p, s) in chain_labels(&ls) { label_of.entry(p).or_insert(s); }

    // sequences: a label that doesn't follow from the one before (by as many steps as pages) starts a new one
    let mut sequences: Vec<Value> = Vec::new();
    let mut seq_of: BTreeMap<i64, usize> = BTreeMap::new();
    let mut prev: Option<(i64, Val)> = None;
    for (&p, s) in &label_of {
        let v = match value(s) { Some(v) => v, None => continue };
        let cont = matches!(prev, Some((q, a)) if fits(a, q, v, p));
        if !cont {
            let style = match v { Val::Arabic(_) => "arabic", Val::Roman(_) => "roman", Val::Chapter(..) => "chapter" };
            sequences.push(json!({"id": sequences.len(), "style": style, "first_page": p, "last_page": p, "first_label": s, "last_label": s}));
        }
        let last = sequences.last_mut().unwrap();
        last["last_page"] = json!(p);
        last["last_label"] = json!(s);
        seq_of.insert(p, sequences.len() - 1);
        prev = Some((p, v));
    }

    // duplicates: the same body lines as an earlier page
    let mut bodies: BTreeMap<i64, Vec<&str>> = BTreeMap::new();
    for l in &ls { if l.band == "body" && !marks.contains_key(&l.text) { bodies.entry(l.page).or_default().push(&l.text); } }
    let mut dup_of: BTreeMap<i64, i64> = BTreeMap::new();
    for (&p, b) in &bodies {
        if b.len() < DUP_LINES { continue; }
        if let Some((&q, _)) = bodies.iter().find(|(&q, c)| q < p && *c == b) { dup_of.insert(p, q); }
    }

    let label_group = best.as_ref().map(|b| b.0);
    let run_json: Vec<Value> = running.iter().enumerate().map(|(id, (key, band, idx))| {
        let l = &ls[idx[0]];
        json!({"id": id, "where": band, "text": l.text, "pattern": key, "box": l.b.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>(),
               "pages": idx.iter().map(|&i| ls[i].page).collect::<Vec<_>>(), "label": Some(id) == label_group})
    }).collect();
    let page_json: Vec<Value> = pages.iter().map(|&p| {
        let on: Vec<usize> = running.iter().enumerate().filter(|(_, r)| r.2.iter().any(|&i| ls[i].page == p)).map(|(id, _)| id).collect();
        let from = best.as_ref().filter(|b| b.2.contains_key(&p)).map(|_| json!(label_group)).unwrap_or(if label_of.contains_key(&p) { json!("chain") } else { Value::Null });
        json!({"n": p, "label": label_of.get(&p), "label_from": from, "sequence": seq_of.get(&p),
               "running": on, "duplicate_of": dup_of.get(&p)})
    }).collect();
    let wm: Vec<Value> = marks.iter().map(|(t, ps)| json!({"text": t, "pages": ps})).collect();
    json!({"running": run_json, "pages": page_json, "sequences": sequences, "watermarks": wm})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_digits_and_roman_numerals() {
        assert_eq!(mask("Page 3 of 20"), "Page # of #");
        assert_eq!(mask("- iv -"), "- # -");
        assert_eq!(mask("Report 6507/7-10"), "Report #/#-#");
        assert_eq!(mask("mix dim civil civ"), "mix dim civil civ");
        assert_eq!(roman("XIV"), Some(14));
        assert_eq!(roman("iiii"), None);
    }

    #[test]
    fn labels_step_by_their_own_style() {
        assert!(steps(value("9").unwrap(), value("10").unwrap()));
        assert!(steps(value("iii").unwrap(), value("iv").unwrap()));
        assert!(steps(value("2-7").unwrap(), value("2-8").unwrap()) && steps(value("2-8").unwrap(), value("3-1").unwrap()));
        assert!(!steps(value("iv").unwrap(), value("1").unwrap()));
    }

    fn doc(pages: &[(&str, &str, &str)]) -> Value {
        // (header, body, footer) per page, on a 600 x 800 page
        let ps: Vec<Value> = pages.iter().enumerate().map(|(i, (h, b, ft))| {
            let mut words = Vec::new();
            for (line, (text, y)) in [(*h, 40.0), (*b, 300.0), (*b, 320.0), (*b, 340.0), (*ft, 770.0)].iter().enumerate() {
                let mut x = 72.0;
                for t in text.split_whitespace() {
                    words.push(json!({"t": t, "x0": x, "y0": y, "x1": x + 8.0 * t.len() as f64, "y1": y + 10.0, "line": line, "size": 10}));
                    x += 8.0 * t.len() as f64 + 4.0;
                }
            }
            json!({"n": i + 1, "width": 600, "height": 800, "words": words})
        }).collect();
        json!({"status": "ok", "pages": ps})
    }

    #[test]
    fn labels_that_alternate_sides_are_chained() {
        let d = doc(&[("Report", "a b c", "iii |"), ("Report", "d e f", "| iv"), ("Report", "g h i", "v |"),
                      ("Report", "j k l", "| vi"), ("Report", "m n o", "vii |")]);
        let labels: Vec<Value> = analyse(&d)["pages"].as_array().unwrap().iter().map(|p| p["label"].clone()).collect();
        assert_eq!(labels, vec![json!("iii"), json!("iv"), json!("v"), json!("vi"), json!("vii")]);
    }

    #[test]
    fn finds_running_lines_labels_and_sequences() {
        let d = doc(&[("Annual Review", "first page words", "- i -"), ("Annual Review", "second page words", "- ii -"),
                      ("Annual Review", "third page words", "- 1 -"), ("Annual Review", "fourth page words", "- 2 -"),
                      ("Annual Review", "fourth page words", "- 3 -")]);
        let out = analyse(&d);
        let run = out["running"].as_array().unwrap();
        assert_eq!(run.len(), 2, "{out}");
        let labels: Vec<Value> = out["pages"].as_array().unwrap().iter().map(|p| p["label"].clone()).collect();
        assert_eq!(labels, vec![json!("i"), json!("ii"), json!("1"), json!("2"), json!("3")]);
        let seqs = out["sequences"].as_array().unwrap();
        assert_eq!(seqs.len(), 2);
        assert_eq!((seqs[0]["style"].as_str(), seqs[1]["style"].as_str()), (Some("roman"), Some("arabic")));
        assert_eq!(out["pages"][4]["duplicate_of"], json!(4));
    }
}
