use ahash::AHashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
pub enum Slot {
    Cap(u8),
    Lit(f64),
}

#[derive(Debug, Clone, Copy)]
pub enum Src {
    Shown(u8),
    Fixed(i64),
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Mul(f64),
    Add(f64),
}

#[derive(Debug)]
pub struct Stat {
    pub id: &'static str,
    pub src: Src,
    pub ops: &'static [Op],
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub neg: bool,
}

#[derive(Debug)]
pub struct Template {
    pub key: &'static str,
    pub slots: &'static [Slot],
    pub stats: &'static [Stat],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStat {
    pub id: &'static str,
    pub value: i64,
}

pub fn normalize(line: &str) -> (String, Vec<f64>) {
    let b = line.as_bytes();
    let (mut key, mut nums, mut i) = (String::with_capacity(line.len()), Vec::new(), 0usize);
    while i < b.len() {
        let signed = (b[i] == b'+' || b[i] == b'-') && i + 1 < b.len() && b[i + 1].is_ascii_digit();
        if signed || b[i].is_ascii_digit() {
            let start = i;
            if signed {
                i += 1;
            }
            while i < b.len()
                && (b[i].is_ascii_digit()
                    || (b[i] == b'.' && i + 1 < b.len() && b[i + 1].is_ascii_digit()))
            {
                i += 1;
            }
            nums.push(line[start..i].parse().unwrap_or(0.0));
            key.push('#');
        } else {
            let ch = line[i..].chars().next().unwrap(); // UTF-8 safe
            key.push(ch);
            i += ch.len_utf8();
        }
    }
    (key, nums)
}

pub fn apply(t: &'static Template, nums: &[f64]) -> Option<Vec<ParsedStat>> {
    if t.slots.len() != nums.len() {
        return None;
    }
    let mut caps = [None::<f64>; 8];
    for (slot, &n) in t.slots.iter().zip(nums) {
        match *slot {
            Slot::Lit(l) => {
                if (l - n).abs() > 1e-9 {
                    return None;
                }
            }
            Slot::Cap(i) => match caps[i as usize] {
                Some(prev) if (prev - n).abs() > 1e-9 => return None,
                _ => caps[i as usize] = Some(n),
            },
        }
    }
    let mut out = Vec::with_capacity(t.stats.len());
    for s in t.stats {
        let value = match s.src {
            Src::Fixed(v) => v,
            Src::Shown(i) => {
                let mut v = caps[i as usize]?;
                for op in s.ops {
                    v = match *op {
                        Op::Mul(k) => v * k,
                        Op::Add(k) => v + k,
                    };
                }
                v.round() as i64 // round(), truncate: 10.2 * 60.0 = 611.99999…
            }
        };
        let in_range = s.min.map_or_else(|| true, |m| value >= m)
            && s.max.map_or_else(|| true, |m| value <= m);
        if in_range == s.neg {
            return None;
        }
        out.push(ParsedStat { id: s.id, value });
    }
    Some(out)
}

pub struct StatMatcher {
    by_key: AHashMap<&'static str, Vec<&'static Template>>,
}

impl StatMatcher {
    // pub fn global() -> &'static Self {
    //     static M: OnceLock<StatMatcher> = OnceLock::new();
    //     M.get_or_init(|| {
    //         let mut by_key: AHashMap<&'static str, Vec<&'static Template>> = AHashMap::new();
    //         for t in crate::generated::stat_templates::TEMPLATES {
    //             by_key.entry(t.key).or_default().push(t);
    //         }
    //         StatMatcher { by_key }
    //     })
    // }

    pub fn templates_for(&self, key: &str) -> &[&'static Template] {
        self.by_key.get(key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn parse_all(&self, line: &str) -> Vec<Vec<ParsedStat>> {
        let (key, nums) = normalize(line.trim());
        match self.by_key.get(key.as_str()) {
            Some(ts) => ts.iter().filter_map(|t| apply(t, &nums)).collect(),
            None => Vec::new(),
        }
    }

    pub fn parse_line(&self, line: &str, accept: impl Fn(&str) -> bool) -> Option<Vec<ParsedStat>> {
        self.parse_all(line)
            .into_iter()
            .find(|c| c.iter().all(|s| accept(s.id)))
    }
}
