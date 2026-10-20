use exile_poe2::stat_matcher::{ParsedStat, StatMatcher};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

const DOMAINS: &[&str] = &["item", "desecrated", "flask", "crafted", "misc"];

fn data_dir() -> PathBuf {
    std::env::var("EXILE_DATA_DIR")
        .map(Into::into)
        .unwrap_or_else(|_| "../data".into())
}

fn strip_markup(s: &str) -> String {
    let mut out = String::new();
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

fn num_end(b: &[u8], mut i: usize) -> Option<usize> {
    if i < b.len() && b[i] == b'-' {
        i += 1;
    }
    let d0 = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == d0 {
        return None;
    }
    if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    Some(i)
}

/// "+(6-10)% to Fire Resistance" -> "+6% to Fire Resistance" (pick = 0) / "+10% ..." (pick = 1)
fn roll(text: &str, pick: usize) -> String {
    let b = text.as_bytes();
    let (mut out, mut i) = (String::new(), 0usize);
    while i < b.len() {
        if b[i] == b'('
            && let Some(e1) = num_end(b, i + 1)
        {
            let sep = ["-", "—", "–"]
                .iter()
                .find(|s| text[e1..].starts_with(**s))
                .map(|s| s.len());
            if let Some(sl) = sep
                && let Some(e2) = num_end(b, e1 + sl)
                && e2 < b.len()
                && b[e2] == b')'
            {
                out.push_str(if pick == 0 {
                    &text[i + 1..e1]
                } else {
                    &text[e1 + sl..e2]
                });
                i = e2 + 1;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out.replace("+-", "-")
}

#[test]
fn sample_lines() {
    let m = StatMatcher::global();
    let any = |_: &str| true;

    assert_eq!(
        m.parse_line("10 Life Regeneration per second", any)
            .unwrap(),
        vec![ParsedStat {
            id: "base_life_regeneration_rate_per_minute",
            value: 600
        }]
    );

    let all = m.parse_all("Adds 136 to 228 Fire Damage");
    assert!(
        all.len() >= 2,
        "expecting local and global candidates: {all:?}"
    );
    let local = m
        .parse_line("Adds 136 to 228 Fire Damage", |id| id.starts_with("local_"))
        .unwrap();
    assert_eq!(
        local,
        vec![
            ParsedStat {
                id: "local_minimum_added_fire_damage",
                value: 136
            },
            ParsedStat {
                id: "local_maximum_added_fire_damage",
                value: 228
            },
        ]
    );

    assert_eq!(
        m.parse_line("+20% to Fire Resistance", any).unwrap()[0].value,
        20
    );
    assert_eq!(
        m.parse_line("-5% to Fire Resistance", any).unwrap()[0].value,
        -5
    );

    assert_eq!(
        m.parse_line("12% increased Critical Damage Bonus", any)
            .unwrap()[0]
            .value,
        12
    );
    assert_eq!(
        m.parse_line("12% reduced Critical Damage Bonus", any)
            .unwrap()[0]
            .value,
        -12
    );
    assert_eq!(
        m.parse_line("+25% to Critical Damage Bonus", any).unwrap()[0].id,
        "local_critical_strike_multiplier_+"
    );

    let flag = m
        .parse_line("25% chance to cause Bleeding on Hit", any)
        .unwrap();
    assert_eq!(flag[0].id, "local_chance_to_bleed_on_hit_25%");
    assert_eq!(flag[0].value, 1);
}

#[test]
fn coverage_report() {
    let mods: Value =
        serde_json::from_str(&fs::read_to_string(data_dir().join("mods.json")).unwrap()).unwrap();
    let m = StatMatcher::global();

    let descs: Value = serde_json::from_str(
        &fs::read_to_string(data_dir().join("stat_descriptions.json")).unwrap(),
    )
    .unwrap();
    let described: HashSet<&str> = descs
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|e| {
            e["ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
        })
        .collect();

    let (mut exact, mut near, mut wrong, mut missing, mut hidden) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut miss_ex, mut wrong_ex): (Vec<String>, Vec<String>) = (vec![], vec![]);

    for (name, md) in mods.as_object().unwrap() {
        if !DOMAINS.contains(&md["domain"].as_str().unwrap_or("")) {
            continue;
        }
        let text = md["text"].as_str().unwrap_or("");
        let stats = md["stats"].as_array().unwrap();
        if text.is_empty() || stats.is_empty() {
            continue;
        }
        for pick in 0..2 {
            let rolled = strip_markup(&roll(text, pick));
            let mut got: HashMap<&'static str, Vec<i64>> = HashMap::new();
            let mut feed = |t: &str| {
                for cand in m.parse_all(t) {
                    for s in cand {
                        got.entry(s.id).or_default().push(s.value);
                    }
                }
            };
            feed(&rolled);
            for line in rolled.split('\n') {
                feed(line);
            }
            for s in stats {
                let id = s["id"].as_str().unwrap();
                let want = s[if pick == 0 { "min" } else { "max" }]
                    .as_f64()
                    .unwrap_or(0.0)
                    .round() as i64;
                match got.get(id) {
                    None if !described.contains(id) => hidden += 1,
                    None => {
                        missing += 1;
                        if miss_ex.len() < 15 {
                            miss_ex.push(format!(
                                "{name}: {id}  <- {:?}",
                                rolled.replace('\n', " | ")
                            ));
                        }
                    }
                    Some(v) if v.contains(&want) => exact += 1,
                    Some(v)
                        if v.iter().any(|x| {
                            (x - want).abs() as f64 <= (want.abs() as f64 * 0.02).max(1.0)
                        }) =>
                    {
                        near += 1
                    }
                    Some(v) => {
                        wrong += 1;
                        if wrong_ex.len() < 15 {
                            wrong_ex.push(format!(
                                "{name}: {id} want {want} got {v:?}  <- {:?}",
                                rolled.replace('\n', " | ")
                            ));
                        }
                    }
                }
            }
        }
    }
    let total = exact + near + wrong + missing;
    eprintln!(
        "\nchecks: {total}  exact: {exact}  near(rounding): {near}  wrong value: {wrong}  id not found: {missing}  (+{hidden} hidden stats without description)"
    );
    eprintln!("--- id not found (examples)");
    miss_ex.iter().for_each(|x| eprintln!("  {x}"));
    eprintln!("--- wrong value (examples)");
    wrong_ex.iter().for_each(|x| eprintln!("  {x}"));
    assert!(
        (exact + near) as f64 / total as f64 > 0.95,
        "coverage dropped below 95%"
    );
}
