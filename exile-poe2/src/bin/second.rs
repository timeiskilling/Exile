use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write as _,
    fs,
    path::PathBuf,
};

const DOMAINS: &[&str] = &["item", "desecrated", "flask", "crafted", "misc"];

#[derive(Clone, Copy)]
enum Op {
    Mul(f64),
    Add(f64),
}

fn inverse(h: &str) -> Option<Op> {
    use Op::*;
    Some(match h {
        "negate" => Mul(-1.0),
        "double" => Mul(0.5),
        "negate_and_double" => Mul(-0.5),
        "times_twenty" => Mul(0.05),
        "subtract_one" => Add(1.0),
        "add_one" => Add(-1.0),
        "plus_two_hundred" => Add(-200.0),
        "divide_by_one_hundred_and_negate" => Mul(-100.0),
        "divide_by_twenty_then_double_0dp" => Mul(10.0),
        "deciseconds_to_seconds" => Mul(10.0),
        "divide_by_two_0dp" => Mul(2.0),
        "divide_by_three" => Mul(3.0),
        "divide_by_four" => Mul(4.0),
        "divide_by_five" => Mul(5.0),
        "divide_by_fifteen_0dp" => Mul(15.0),
        "divide_by_fifty" => Mul(50.0),
        h if h.starts_with("per_minute_to_per_second") => Mul(60.0),
        h if h.starts_with("milliseconds_to_seconds") => Mul(1000.0),
        h if h.starts_with("divide_by_one_hundred") => Mul(100.0),
        h if h.starts_with("divide_by_ten") => Mul(10.0),
        _ => return None,
    })
}

/// "[Strength|Strength]" -> "Strength", "[Armour]" -> "Armour"
fn strip_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '[' {
            let mut inner = String::new();
            for c in it.by_ref() {
                if c == ']' {
                    break;
                }
                inner.push(c);
            }
            out.push_str(inner.rsplit('|').next().unwrap());
        } else {
            out.push(c);
        }
    }
    out
}

enum Slot {
    Cap(usize),
    Lit(f64),
}

/// "{0}% increased Damage" -> ("#% increased Damage", [Cap(0)]).
fn compile_template(raw: &str) -> (String, Vec<Slot>) {
    let s = strip_markup(raw);
    let b = s.as_bytes();
    let (mut key, mut slots, mut i) = (String::new(), Vec::new(), 0usize);
    while i < b.len() {
        let c = b[i];
        let signed = (c == b'+' || c == b'-')
            && i + 1 < b.len()
            && (b[i + 1] == b'{' || b[i + 1].is_ascii_digit());
        if signed || c == b'{' || c.is_ascii_digit() {
            let mut j = if signed { i + 1 } else { i };
            if b[j] == b'{' {
                let end = j + s[j..].find('}').expect("unclosed {");
                let idx = s[j + 1..end].split(':').next().unwrap().parse().unwrap();
                slots.push(Slot::Cap(idx));
                j = end + 1;
            } else {
                while j < b.len()
                    && (b[j].is_ascii_digit()
                        || (b[j] == b'.' && j + 1 < b.len() && b[j + 1].is_ascii_digit()))
                {
                    j += 1;
                }
                slots.push(Slot::Lit(s[i..j].parse().unwrap()));
            }
            key.push('#');
            i = j;
        } else {
            let ch = s[i..].chars().next().unwrap(); // UTF-8 safe
            key.push(ch);
            i += ch.len_utf8();
        }
    }
    (key, slots)
}

fn bound(v: &Value) -> Option<i64> {
    v.as_f64().map(|f| f.round() as i64)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let data: PathBuf = args.next().map(Into::into).unwrap_or_else(|| "data".into());
    let out_path: PathBuf = args
        .next()
        .map(Into::into)
        .unwrap_or_else(|| "exile-poe2/src/generated/stat_templates.rs".into());

    let mods: Value =
        serde_json::from_str(&fs::read_to_string(data.join("mods.json")).unwrap()).unwrap();
    let descs: Value =
        serde_json::from_str(&fs::read_to_string(data.join("stat_descriptions.json")).unwrap())
            .unwrap();

    let mut needed: HashSet<&str> = HashSet::new();
    for (_, m) in mods.as_object().unwrap() {
        if !DOMAINS.contains(&m["domain"].as_str().unwrap_or("")) {
            continue;
        }
        for s in m["stats"].as_array().into_iter().flatten() {
            needed.insert(s["id"].as_str().unwrap());
        }
    }

    let mut out = String::from(
        "// @generated\n\
         #![allow(clippy::all)]\n\
         use crate::stat_matcher::{Op, Slot, Src, Stat as S, Template as T};\n\n\
         pub static TEMPLATES: &[T] = &[\n",
    );
    let (mut n_tpl, mut n_skipped) = (0usize, 0usize);
    let (mut lines_t0, mut lines_t1): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    let mut skipped_handlers: BTreeMap<String, usize> = BTreeMap::new();

    for e in descs.as_array().unwrap() {
        let ids: Vec<&str> = e["ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let tier0 = ids.iter().any(|id| needed.contains(id));
        for v in e["English"].as_array().unwrap() {
            let handlers = v["index_handlers"].as_array().unwrap();
            let conds = v["condition"].as_array().unwrap();
            let fmts = v["format"].as_array().unwrap();

            let mut ops_per_stat: Vec<Vec<Op>> = Vec::new();
            let mut opaque = false;
            for hs in handlers {
                let mut ops = Vec::new();
                for h in hs.as_array().unwrap() {
                    let h = h.as_str().unwrap();
                    match inverse(h) {
                        Some(op) => ops.push(op),
                        None => {
                            *skipped_handlers.entry(h.to_string()).or_default() += 1;
                            opaque = true;
                        }
                    }
                }
                ops.reverse();
                ops_per_stat.push(ops);
            }
            if opaque {
                n_skipped += 1;
                continue;
            }

            let (key, slots) = compile_template(v["string"].as_str().unwrap());
            let shown: HashSet<usize> = slots
                .iter()
                .filter_map(|s| if let Slot::Cap(i) = s { Some(*i) } else { None })
                .collect();

            let mut stats = String::new();
            for (i, id) in ids.iter().enumerate() {
                let c = &conds[i];
                let (min, max) = (bound(&c["min"]), bound(&c["max"]));
                let neg = c["negated"].as_bool().unwrap_or(false);
                let src = if fmts[i] == "ignore" || !shown.contains(&i) {
                    let fixed = if !neg {
                        min.or(max).unwrap_or(1)
                    } else {
                        max.map(|m| m + 1).or(min.map(|m| m - 1)).unwrap_or(1)
                    };
                    format!("Src::Fixed({fixed})")
                } else {
                    format!("Src::Shown({i})")
                };
                let ops: Vec<String> = ops_per_stat[i]
                    .iter()
                    .map(|o| match o {
                        Op::Mul(k) => format!("Op::Mul({k:?})"),
                        Op::Add(k) => format!("Op::Add({k:?})"),
                    })
                    .collect();
                write!(
                    stats,
                    "S{{id:{id:?},src:{src},ops:&[{}],min:{min:?},max:{max:?},neg:{neg}}},",
                    ops.join(",")
                )
                .unwrap();
            }
            let slots: Vec<String> = slots
                .iter()
                .map(|s| match s {
                    Slot::Cap(i) => format!("Slot::Cap({i})"),
                    Slot::Lit(f) => format!("Slot::Lit({f:?})"),
                })
                .collect();
            let line = format!(
                "T{{key:{key:?},slots:&[{}],stats:&[{stats}]}},\n",
                slots.join(",")
            );
            if tier0 {
                lines_t0.push(line)
            } else {
                lines_t1.push(line)
            }
            n_tpl += 1;
        }
    }
    let n_t0 = lines_t0.len();
    out.extend(lines_t0);
    out.extend(lines_t1);
    out.push_str("];\n");

    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    fs::write(&out_path, out).unwrap();
    eprintln!("stat ids in item-domain mods : {}", needed.len());
    eprintln!("templates written            : {n_tpl}  (tier0: {n_t0})");
    eprintln!("templates skipped (opaque)   : {n_skipped}  {skipped_handlers:?}");
    eprintln!("-> {}", out_path.display());
}
