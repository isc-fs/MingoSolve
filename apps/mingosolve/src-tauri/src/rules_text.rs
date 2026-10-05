//! Rulebook text: per-page PDF extraction, splitting into rule entries by rule id (`T 2.9.2`, `EV 5.1.3`...), and an
//! in-memory token index with ranked search. Lives in the app crate so the PDF dependencies stay out of the engine
//! and the CLI. The rulebook is copyrighted: nothing here is bundled, only what the user loads is indexed.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// One rule (or section heading) of the rulebook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    /// Title of the section heading this rule sits under (the heading's own text for headings).
    pub section: String,
    pub text: String,
    /// 1-based PDF page where the entry starts.
    pub page: u32,
    pub heading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rulebook {
    pub pages: u32,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hit {
    pub id: String,
    pub title: String,
    pub page: u32,
    pub snippet: String,
    /// `[start, end)` of the matched words inside `snippet`, in UTF-16 units (what JS string indices use).
    pub matches: Vec<[usize; 2]>,
}

/// Text of every page of a PDF, in order.
pub fn extract_pages(path: &Path) -> Result<Vec<String>, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("could not open {}: {e}", path.display()))?;
    extract_pages_mem(&bytes)
}

pub fn extract_pages_mem(pdf: &[u8]) -> Result<Vec<String>, String> {
    pdf_extract::extract_text_from_mem_by_pages(pdf)
        .map_err(|e| format!("could not read the PDF: {e}"))
}

fn fix_ligatures(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\u{FB00}' => out.push_str("ff"),
            '\u{FB01}' => out.push_str("fi"),
            '\u{FB02}' => out.push_str("fl"),
            '\u{FB03}' => out.push_str("ffi"),
            '\u{FB04}' => out.push_str("ffl"),
            '\u{FB05}' | '\u{FB06}' => out.push_str("st"),
            '\u{A0}' | '\u{2009}' | '\u{202F}' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

fn is_unit_char(t: &str) -> bool {
    let mut cs = t.chars();
    matches!((cs.next(), cs.next()), (Some(c), None) if c.is_alphanumeric() || "°%²³".contains(c))
}

/// The PDF puts a space between every glyph of some runs ("7 5 %", "1 5 2 5 m m"): glue digit runs and unit letters.
fn fix_spacing(line: &str) -> String {
    let toks: Vec<&str> = line.split(' ').filter(|t| !t.is_empty()).collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if !is_unit_char(toks[i]) {
            out.push(toks[i].to_string());
            i += 1;
            continue;
        }
        let mut j = i;
        while j < toks.len() && is_unit_char(toks[j]) {
            j += 1;
        }
        let run = &toks[i..j];
        let mut after_digit = out
            .last()
            .is_some_and(|p| p.chars().last().is_some_and(|c| c.is_ascii_digit()));
        let digit = |t: &str| t.chars().all(|c| c.is_ascii_digit());
        let letter = |t: &str| t.chars().all(|c| c.is_alphabetic());
        let mut k = 0;
        while k < run.len() {
            let mut e = k + 1;
            if digit(run[k]) {
                while e < run.len() && digit(run[e]) {
                    e += 1;
                }
            } else if letter(run[k]) {
                while e < run.len() && letter(run[e]) {
                    e += 1;
                }
            }
            let n = e - k;
            let glue = if digit(run[k]) {
                n >= 2
            } else if letter(run[k]) {
                // "z o n e" (3+ letters), or a unit after a number ("m m")
                n >= 3 || (n >= 2 && after_digit)
            } else {
                false
            };
            if glue {
                out.push(run[k..e].concat());
            } else {
                out.extend(run[k..e].iter().map(|s| s.to_string()));
            }
            after_digit = digit(run[k]);
            k = e;
        }
        i = j;
    }
    let mut s = out.join(" ");
    // numbers split around their decimal separator, and punctuation floating off its neighbour
    for (from, to) in [(" . ", "."), (" , ", ",")] {
        let mut res = String::with_capacity(s.len());
        let cs: Vec<char> = s.chars().collect();
        let f: Vec<char> = from.chars().collect();
        let mut x = 0;
        while x < cs.len() {
            let is_sep = x + 3 <= cs.len()
                && cs[x..x + 3] == f[..]
                && x > 0
                && cs[x - 1].is_ascii_digit()
                && cs.get(x + 3).is_some_and(|c| c.is_ascii_digit());
            if is_sep {
                res.push_str(to);
                x += 3;
            } else {
                res.push(cs[x]);
                x += 1;
            }
        }
        s = res;
    }
    for (from, to) in [
        (" .", "."),
        (" ,", ","),
        (" ;", ";"),
        (" :", ":"),
        (" )", ")"),
        ("( ", "("),
        (" ]", "]"),
        ("[ ", "["),
    ] {
        s = s.replace(from, to);
    }
    s
}

fn clean_line(raw: &str) -> String {
    let s = fix_ligatures(raw);
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    fix_spacing(&s)
}

/// Digits folded, so "Version 1.0  23 / 115" and "... 24 / 115" are the same line.
fn repeat_key(line: &str) -> String {
    let mut k = String::new();
    for c in line.chars() {
        if c.is_ascii_digit() {
            if !k.ends_with('#') {
                k.push('#');
            }
        } else {
            k.push(c);
        }
    }
    k
}

/// Lines that open or close at least half the pages (running headers, footers, page numbers).
fn running_lines(pages: &[Vec<String>]) -> HashSet<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for p in pages {
        let lines: Vec<&String> = p.iter().filter(|l| !l.is_empty()).collect();
        let edge: HashSet<String> = lines
            .iter()
            .take(2)
            .chain(lines.iter().rev().take(2))
            .map(|l| repeat_key(l))
            .collect();
        for k in edge {
            *seen.entry(k).or_default() += 1;
        }
    }
    let need = pages.len().div_ceil(2).max(2);
    seen.into_iter()
        .filter(|(_, n)| *n >= need)
        .map(|(k, _)| k)
        .collect()
}

struct Start {
    prefix: String,
    nums: Vec<u32>,
    rest: String,
}

/// `T 2.9.2 The smaller track...` -> prefix `T`, numbers [2,9,2], the rest. Only ids with two or more numbers.
fn parse_start(line: &str) -> Option<Start> {
    let (prefix, tail) = line.split_once(' ')?;
    if prefix.is_empty() || prefix.len() > 3 || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    let (id, rest) = tail.split_once(' ').unwrap_or((tail, ""));
    let nums: Option<Vec<u32>> = id.split('.').map(|n| n.parse().ok()).collect();
    let nums = nums?;
    if nums.len() < 2 || id.ends_with('.') || id.starts_with('.') {
        return None;
    }
    let first = rest.chars().next()?;
    // lowercase openings are definitions ("disqualified (DQ) — being removed ...")
    let definition = rest.find('—').is_some_and(|p| p < 80);
    if !(first.is_uppercase() || first.is_ascii_digit() || "[“\"▪(".contains(first) || definition)
    {
        return None;
    }
    // changelog rows: "EV 5.5.13 1.0 Extended ..."
    let mut w = rest.split(' ');
    if let (Some(v), Some(next)) = (w.next(), w.next()) {
        let isver = v
            .split_once('.')
            .is_some_and(|(a, b)| a.parse::<u32>().is_ok() && b.parse::<u32>().is_ok());
        if isver && next.chars().next().is_some_and(char::is_uppercase) {
            return None;
        }
    }
    Some(Start {
        prefix: prefix.to_string(),
        nums,
        rest: rest.to_string(),
    })
}

fn is_chapter_line(line: &str) -> bool {
    let mut it = line.splitn(2, ' ');
    let (Some(p), Some(rest)) = (it.next(), it.next()) else {
        return false;
    };
    let (letters, digits): (String, String) = if p.chars().any(|c| c.is_ascii_digit()) {
        let split = p.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
        (p[..split].to_string(), p[split..].to_string())
    } else {
        let (d, rest2) = rest.split_once(' ').unwrap_or((rest, ""));
        return p.len() <= 3
            && p.chars().all(|c| c.is_ascii_uppercase())
            && d.len() <= 2
            && !d.is_empty()
            && d.chars().all(|c| c.is_ascii_digit())
            && chapter_title(rest2);
    };
    !letters.is_empty()
        && letters.len() <= 3
        && letters.chars().all(|c| c.is_ascii_uppercase())
        && digits.len() <= 2
        && digits.chars().all(|c| c.is_ascii_digit())
        && chapter_title(rest)
}

fn chapter_title(t: &str) -> bool {
    t.chars().next().is_some_and(char::is_uppercase)
        && !t.ends_with('.')
        && t.split(' ').count() <= 12
}

fn id_string(prefix: &str, nums: &[u32]) -> String {
    let n: Vec<String> = nums.iter().map(u32::to_string).collect();
    format!("{prefix} {}", n.join("."))
}

/// Hyphenated compounds written whole inside a line ("high-voltage"), to tell them from line-end hyphenation.
fn compounds(lines: &[(u32, String)]) -> HashSet<String> {
    let mut set = HashSet::new();
    for (_, l) in lines {
        for w in l.split(' ') {
            if let Some((a, b)) = w.split_once('-') {
                if !a.is_empty()
                    && !b.is_empty()
                    && !b.contains('-')
                    && a.chars().all(char::is_lowercase)
                    && b.chars().all(char::is_lowercase)
                {
                    set.insert(w.to_string());
                }
            }
        }
    }
    set
}

fn last_word(s: &str) -> &str {
    let t = s.trim_end_matches(|c: char| !c.is_alphabetic());
    let start = t
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_alphabetic())
        .map_or(0, |(i, c)| i + c.len_utf8());
    &t[start..]
}

fn join_lines(lines: &[String], comp: &HashSet<String>) -> String {
    let mut out = String::new();
    for line in lines {
        if out.is_empty() {
            out.push_str(line);
            continue;
        }
        let next_lower = line.chars().next().is_some_and(char::is_lowercase);
        if let Some(stem) = out.strip_suffix('\u{AD}') {
            out = format!("{stem}{line}");
        } else if next_lower && out.ends_with('-') {
            let body = out[..out.len() - 1].trim_end().to_string();
            let attached = out.len() - 1 == body.len();
            let left = last_word(&body);
            let right: String = line.chars().take_while(|c| c.is_alphabetic()).collect();
            let lower_left = body.chars().last().is_some_and(char::is_lowercase);
            let keep = !lower_left || comp.contains(&format!("{left}-{right}"));
            if attached || lower_left {
                out = format!("{body}{}{line}", if keep { "-" } else { "" });
            } else {
                out = format!("{out} {line}");
            }
        } else {
            out.push(' ');
            out.push_str(line);
        }
    }
    out
}

/// Splits per-page text into rule entries. Running headers/footers and chapter title lines are dropped.
pub fn build_rulebook(pages: &[String]) -> Rulebook {
    let cleaned: Vec<Vec<String>> = pages
        .iter()
        .map(|p| p.lines().map(clean_line).collect())
        .collect();
    let running = running_lines(&cleaned);

    // (page, line); a blank line is kept as "" so rule boundaries stay visible
    let mut lines: Vec<(u32, String)> = Vec::new();
    for (i, p) in cleaned.iter().enumerate() {
        for l in p {
            if l.is_empty() {
                lines.push((i as u32 + 1, String::new()));
            } else if (parse_start(l).is_some() || !running.contains(&repeat_key(l)))
                && !is_chapter_line(l)
                && !l.contains(". . . .")
            {
                lines.push((i as u32 + 1, l.clone()));
            }
        }
    }
    let comp = compounds(&lines);

    struct Open {
        prefix: String,
        nums: Vec<u32>,
        page: u32,
        lines: Vec<String>,
    }
    let mut done: Vec<(Open, String)> = Vec::new();
    let mut cur: Option<Open> = None;
    let mut last: HashMap<String, Vec<u32>> = HashMap::new();
    let mut prev_ends = true;

    for (page, l) in &lines {
        if l.is_empty() {
            prev_ends = true;
            continue;
        }
        let start = parse_start(l).filter(|s| {
            prev_ends
                && last
                    .get(&s.prefix)
                    .is_none_or(|prev| s.nums.as_slice() > prev.as_slice())
        });
        if let Some(s) = start {
            if let Some(o) = cur.take() {
                let text = join_lines(&o.lines, &comp);
                done.push((o, text));
            }
            last.insert(s.prefix.clone(), s.nums.clone());
            cur = Some(Open {
                prefix: s.prefix,
                nums: s.nums,
                page: *page,
                lines: vec![s.rest],
            });
        } else if let Some(o) = cur.as_mut() {
            o.lines.push(l.clone());
        }
        prev_ends = l.ends_with(['.', ':', ';', ')', '?']) || l.starts_with('▪');
    }
    if let Some(o) = cur.take() {
        let text = join_lines(&o.lines, &comp);
        done.push((o, text));
    }

    let titles: HashMap<String, String> = done
        .iter()
        .filter(|(o, t)| o.nums.len() == 2 && !t.ends_with('.'))
        .map(|(o, t)| (id_string(&o.prefix, &o.nums), t.clone()))
        .collect();
    let entries = done
        .into_iter()
        .map(|(o, text)| {
            let id = id_string(&o.prefix, &o.nums);
            let heading = titles.contains_key(&id);
            let section = if heading {
                text.clone()
            } else {
                titles
                    .get(&id_string(&o.prefix, &o.nums[..2]))
                    .cloned()
                    .unwrap_or_default()
            };
            Entry {
                id,
                section,
                text,
                page: o.page,
                heading,
            }
        })
        .collect();
    Rulebook {
        pages: pages.len() as u32,
        entries,
    }
}

const ALIASES: [(&str, &str); 6] = [
    ("tyre", "tire"),
    ("centre", "center"),
    ("colour", "color"),
    ("metre", "meter"),
    ("litre", "liter"),
    ("grey", "gray"),
];

fn norm_word(w: &str) -> String {
    let mut w = w.to_lowercase();
    for (uk, us) in ALIASES {
        if let Some(rest) = w.strip_prefix(uk) {
            w = format!("{us}{rest}");
            break;
        }
    }
    if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") {
        w.pop();
    }
    w
}

/// Word spans of `s` as (byte start, byte end, normalised word). Dots and commas inside numbers stay in the word.
fn words(s: &str) -> Vec<(usize, usize, String)> {
    let cs: Vec<(usize, char)> = s.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        if !cs[i].1.is_alphanumeric() {
            i += 1;
            continue;
        }
        let start = cs[i].0;
        let mut j = i;
        while j < cs.len() {
            let c = cs[j].1;
            let num_sep = (c == '.' || c == ',')
                && j > i
                && cs[j - 1].1.is_ascii_digit()
                && cs.get(j + 1).is_some_and(|n| n.1.is_ascii_digit());
            if c.is_alphanumeric() || num_sep {
                j += 1;
            } else {
                break;
            }
        }
        let end = cs.get(j).map_or(s.len(), |c| c.0);
        out.push((start, end, norm_word(&s[start..end])));
        i = j;
    }
    out
}

fn tokens(s: &str) -> Vec<String> {
    words(s).into_iter().map(|w| w.2).collect()
}

fn first_line(text: &str) -> &str {
    let t = text.trim();
    let cut = t
        .char_indices()
        .take(80)
        .last()
        .map_or(0, |(i, c)| i + c.len_utf8());
    &t[..cut]
}

struct Indexed {
    title: HashSet<String>,
    body: HashMap<String, u32>,
    len: usize,
}

pub struct Index {
    pub book: Rulebook,
    docs: Vec<Indexed>,
    inverted: BTreeMap<String, Vec<u32>>,
    by_id: HashMap<String, usize>,
}

impl Index {
    pub fn new(book: Rulebook) -> Index {
        let mut docs = Vec::with_capacity(book.entries.len());
        let mut inverted: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        let mut by_id = HashMap::new();
        for (n, e) in book.entries.iter().enumerate() {
            let mut title: HashSet<String> = tokens(&e.section).into_iter().collect();
            title.extend(tokens(first_line(&e.text)));
            let mut body: HashMap<String, u32> = HashMap::new();
            let toks = tokens(&e.text);
            let len = toks.len();
            for t in toks {
                *body.entry(t).or_default() += 1;
            }
            for t in title.iter().chain(body.keys()) {
                let v = inverted.entry(t.clone()).or_default();
                if v.last() != Some(&(n as u32)) {
                    v.push(n as u32);
                }
            }
            by_id.insert(e.id.clone(), n);
            docs.push(Indexed { title, body, len });
        }
        Index {
            book,
            docs,
            inverted,
            by_id,
        }
    }

    /// Ranked: exact rule id (then its children), all words in the title/first line, all words in the body, partial.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Hit> {
        let q = query.trim();
        if q.is_empty() || limit == 0 {
            return vec![];
        }
        let mut scored: Vec<(f64, usize)> = Vec::new();
        let mut taken: HashSet<usize> = HashSet::new();

        if let Some(cid) = query_id(q) {
            if let Some(&n) = self.by_id.get(&cid) {
                scored.push((10_000.0, n));
                taken.insert(n);
            }
            let dotted = format!("{cid}.");
            for (n, e) in self.book.entries.iter().enumerate() {
                if !taken.contains(&n) && e.id.starts_with(&dotted) {
                    scored.push((9_000.0 - n as f64 * 1e-3, n));
                    taken.insert(n);
                }
            }
        }

        let qt = tokens(q);
        if !qt.is_empty() {
            let mut cand: HashMap<usize, usize> = HashMap::new();
            for t in &qt {
                let mut hit: HashSet<u32> = HashSet::new();
                if let Some(v) = self.inverted.get(t) {
                    hit.extend(v);
                }
                if t.len() >= 3 {
                    for (k, v) in self.inverted.range(t.clone()..) {
                        if !k.starts_with(t.as_str()) {
                            break;
                        }
                        hit.extend(v);
                    }
                }
                for n in hit {
                    *cand.entry(n as usize).or_default() += 1;
                }
            }
            for (n, matched) in cand {
                if taken.contains(&n) {
                    continue;
                }
                scored.push((self.score(n, &qt, matched), n));
            }
        }
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, n)| self.hit(n, &qt))
            .collect()
    }

    fn has(&self, n: usize, t: &str, title_only: bool) -> bool {
        let d = &self.docs[n];
        let exact = d.title.contains(t) || (!title_only && d.body.contains_key(t));
        if exact || t.len() < 3 {
            return exact;
        }
        let pre = |k: &String| k.starts_with(t);
        d.title.iter().any(pre) || (!title_only && d.body.keys().any(pre))
    }

    fn score(&self, n: usize, qt: &[String], matched: usize) -> f64 {
        let d = &self.docs[n];
        let in_title = qt.iter().filter(|t| self.has(n, t, true)).count();
        let in_any = qt.iter().filter(|t| self.has(n, t, false)).count();
        let all = qt.len();
        let tf: f64 = qt
            .iter()
            .map(|t| f64::from(*d.body.get(t).unwrap_or(&0)).min(5.0))
            .sum();
        let short = 1.0 / (1.0 + d.len as f64 / 60.0);
        let phrase = if all > 1 && self.phrase(n, qt) {
            40.0
        } else {
            0.0
        };
        let base = if in_title == all {
            4_000.0
        } else if in_any == all {
            2_000.0
        } else {
            500.0 * matched as f64 / all as f64
        };
        let heading = if self.book.entries[n].heading {
            30.0
        } else {
            0.0
        };
        base + phrase + heading + tf * 4.0 + short * 20.0 + in_title as f64 * 60.0
    }

    /// The query words appear consecutively in the entry.
    fn phrase(&self, n: usize, qt: &[String]) -> bool {
        let e = &self.book.entries[n];
        let toks = tokens(&e.text);
        toks.windows(qt.len()).any(|w| {
            w.iter()
                .zip(qt)
                .all(|(a, b)| a == b || a.starts_with(b.as_str()))
        })
    }

    fn hit(&self, n: usize, qt: &[String]) -> Hit {
        let e = &self.book.entries[n];
        let (snippet, matches) = snippet(&e.text, qt, 300);
        let title = if e.section.is_empty() {
            first_line(&e.text).to_string()
        } else {
            e.section.clone()
        };
        Hit {
            id: e.id.clone(),
            title,
            page: e.page,
            snippet,
            matches,
        }
    }
}

/// `T 2.9.2`, `t2.9.2`, `T2.9.2` -> `T 2.9.2`; anything else -> None.
fn query_id(q: &str) -> Option<String> {
    let q = q.trim().trim_end_matches('.');
    let split = q.find(|c: char| c.is_ascii_digit())?;
    let (letters, nums) = q.split_at(split);
    let letters = letters.trim();
    if letters.is_empty()
        || letters.len() > 3
        || !letters.chars().all(|c| c.is_ascii_alphabetic())
        || !nums
            .split('.')
            .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    Some(format!("{} {nums}", letters.to_ascii_uppercase()))
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// Up to `max` chars of `text` around the first matched word, with the UTF-16 spans of every matched word.
fn snippet(text: &str, qt: &[String], max: usize) -> (String, Vec<[usize; 2]>) {
    let ws = words(text);
    let is_match = |w: &str| {
        qt.iter()
            .any(|t| w == t || (t.len() >= 3 && w.starts_with(t.as_str())))
    };
    let first = ws.iter().find(|w| is_match(&w.2)).map(|w| w.0);
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let total = chars.len();
    let (mut from, mut to) = (0, total.min(max));
    if let Some(f) = first {
        let at = chars.iter().position(|c| c.0 == f).unwrap_or(0);
        if at > max / 3 {
            from = at - max / 3;
            to = (from + max).min(total);
        }
    }
    if from > 0 {
        while from < to && chars[from - 1].1 != ' ' {
            from += 1;
        }
    }
    if to < total {
        while to > from && chars[to].1 != ' ' {
            to -= 1;
        }
    }
    let b0 = chars.get(from).map_or(text.len(), |c| c.0);
    let b1 = chars.get(to).map_or(text.len(), |c| c.0);
    let body = text[b0..b1].trim();
    let b0 = b0 + (text[b0..b1].len() - text[b0..b1].trim_start().len());
    let prefix = if from > 0 { "… " } else { "" };
    let suffix = if to < total { " …" } else { "" };
    let snippet = format!("{prefix}{body}{suffix}");
    let off = utf16_len(prefix);
    let matches = ws
        .iter()
        .filter(|w| w.0 >= b0 && w.1 <= b0 + body.len() && is_match(&w.2))
        .map(|w| {
            let s = off + utf16_len(&text[b0..w.0]);
            [s, s + utf16_len(&text[w.0..w.1])]
        })
        .collect();
    (snippet, matches)
}

#[cfg(test)]
pub(crate) mod tests;
