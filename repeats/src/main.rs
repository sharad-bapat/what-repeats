//! repeats-cli: one JSON line in for each file, wordbox-cli's output, one JSON line out.
//!   wordbox-cli report.pdf | repeats-cli -
//!   repeats-cli report.wordbox.json [more.json ...]
use std::io::{BufRead, Read};

fn one(line: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(doc) => {
            let mut out = repeats::analyse(&doc);
            out["file"] = doc.get("file").cloned().unwrap_or(serde_json::Value::Null);
            out.to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["-"] {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            if !line.trim().is_empty() { println!("{}", one(&line)); }
        }
        return;
    }
    for path in args {
        let mut s = String::new();
        match std::fs::File::open(&path).and_then(|mut f| f.read_to_string(&mut s)) {
            Ok(_) => for line in s.lines().filter(|l| !l.trim().is_empty()) { println!("{}", one(line)); },
            Err(e) => println!("{}", serde_json::json!({"file": path, "error": e.to_string()})),
        }
    }
}
