//! Rulebook text tests: a synthetic PDF (made-up rules, built here with lopdf) pins extraction, splitting, header
//! and footer stripping, de-hyphenation and ranking; the real FS-Rules PDF is checked only when `FSQ_RULES_PDF` is set.

use super::*;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

/// A PDF with one text line per `(y, text)` on each page, in Helvetica 10 pt.
pub fn synthetic_pdf(pages: &[Vec<(f32, &str)>]) -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
    });
    let resources = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font } });
    let mut kids = vec![];
    for lines in pages {
        let mut ops = vec![Operation::new("BT", vec![])];
        ops.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));
        let mut prev = 0.0;
        for (y, text) in lines {
            ops.push(Operation::new(
                "Td",
                vec![
                    (if prev == 0.0 { 60.0 } else { 0.0 }).into(),
                    (*y - if prev == 0.0 { 0.0 } else { prev }).into(),
                ],
            ));
            ops.push(Operation::new("Tj", vec![Object::string_literal(*text)]));
            prev = *y;
        }
        ops.push(Operation::new("ET", vec![]));
        let content = doc.add_object(Stream::new(
            dictionary! {},
            Content { operations: ops }.encode().unwrap(),
        ));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        kids.push(page.into());
    }
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => kids, "Count" => pages.len() as i64, "Resources" => resources,
        }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

const HEADER: &str = "Synthetic Rules 2099";

fn page1() -> Vec<(f32, &'static str)> {
    vec![
        (800.0, HEADER),
        (760.0, "Z 1 Wheels and Lights"),
        (730.0, "Z 1.2 Wheel Fastening"),
        (
            710.0,
            "Z 1.2.1 Every wheel must be retained by a nut of at least",
        ),
        (698.0, "twelve millimetres in diameter."),
        (
            676.0,
            "Z 1.2.2 The retention device must be visible from the outside and be demon-",
        ),
        (
            664.0,
            "strated at inspection. Cables are high-voltage and must be orange; a high-",
        ),
        (652.0, "voltage cable must never touch a wheel."),
        (
            630.0,
            "Z 1.2.3 A 1 5 2 5 m m track is the smallest allowed by the sparkle committee.",
        ),
        (30.0, "Synthetic Rules 2099 Version 0.1 1 / 2"),
    ]
}

fn page2() -> Vec<(f32, &'static str)> {
    vec![
        (800.0, HEADER),
        (780.0, "Z 3 Lights"),
        (750.0, "Z 3.1 Beacon Light"),
        (
            730.0,
            "Z 3.1.1 The device must be green when the sparkle system is idle.",
        ),
        (
            708.0,
            "Z 3.1.2 A flashing red signal means the sparkle system is active, see Z 1.2.1.",
        ),
        (
            686.0,
            "Z 3.2.1 Wheel nuts must be painted orange, which makes them visible to the marshals near the beacon.",
        ),
        (30.0, "Synthetic Rules 2099 Version 0.1 2 / 2"),
    ]
}

fn synthetic_pages() -> Vec<String> {
    extract_pages_mem(&synthetic_pdf(&[page1(), page2()])).unwrap()
}

fn book() -> Rulebook {
    build_rulebook(&synthetic_pages())
}

fn ids(b: &Rulebook) -> Vec<&str> {
    b.entries.iter().map(|e| e.id.as_str()).collect()
}

fn entry<'a>(b: &'a Rulebook, id: &str) -> &'a Entry {
    b.entries
        .iter()
        .find(|e| e.id == id)
        .unwrap_or_else(|| panic!("no {id} in {:?}", ids(b)))
}

#[test]
fn pdf_pages_and_rules_are_split_by_id() {
    let pages = synthetic_pages();
    assert_eq!(pages.len(), 2);
    let b = build_rulebook(&pages);
    assert_eq!(b.pages, 2);
    assert_eq!(
        ids(&b),
        ["Z 1.2", "Z 1.2.1", "Z 1.2.2", "Z 1.2.3", "Z 3.1", "Z 3.1.1", "Z 3.1.2", "Z 3.2.1"]
    );
    let multi = entry(&b, "Z 1.2.1");
    assert_eq!(
        multi.text,
        "Every wheel must be retained by a nut of at least twelve millimetres in diameter."
    );
    assert_eq!((multi.page, entry(&b, "Z 3.1.1").page), (1, 2));
    assert!(entry(&b, "Z 1.2").heading && !multi.heading);
    assert_eq!(multi.section, "Wheel Fastening");
    // a cross-reference inside a line must not start a new rule
    assert!(entry(&b, "Z 3.1.2").text.ends_with("see Z 1.2.1."));
}

#[test]
fn running_headers_footers_and_chapter_titles_are_stripped() {
    let b = book();
    for e in &b.entries {
        for junk in [
            HEADER,
            "Version 0.1",
            "1 / 2",
            "Wheels and Lights",
            "Z 3 Lights",
        ] {
            assert!(
                !e.text.contains(junk),
                "{} still holds {junk:?}: {}",
                e.id,
                e.text
            );
        }
    }
    // the last rule of page 1 is followed by the footer on the page; its text must stop at its own sentence
    assert!(entry(&b, "Z 1.2.3").text.ends_with("committee."));
}

#[test]
fn line_end_hyphenation_is_undone_but_real_compounds_survive() {
    let b = book();
    let t = &entry(&b, "Z 1.2.2").text;
    assert!(t.contains("be demonstrated at inspection."), "{t}");
    assert!(!t.contains("demon-"));
    // "high-voltage" is written whole elsewhere, so a break inside it keeps the hyphen
    assert!(t.contains("a high-voltage cable"), "{t}");
}

#[test]
fn spaced_out_glyphs_and_ligatures_are_repaired() {
    // shapes seen in the real rulebook: "1 5 2 5 m m", "7 5 %", "( TSAL )", "2 . 4 m m", and fi/fl ligatures
    let b = build_rulebook(&[
        "T 2.9 Wheelbase\n\nT 2.9.1 A wheelbase of  1 5 2 5 m m  and  7 5 %  of the track ( TSAL ) , 2 . 4 m m deep, oﬃcial ﬂag.\n".into(),
    ]);
    let t = &entry(&b, "T 2.9.1").text;
    assert_eq!(
        t,
        "A wheelbase of 1525 mm and 75 % of the track (TSAL), 2.4 mm deep, official flag."
    );
}

#[test]
fn changelog_rows_and_toc_lines_do_not_become_rules() {
    let b = build_rulebook(&[
        "Changelog\n\nT 2.9.2 1.0 Clarified rule\n\nT 2 General Design . . . . . . . . 21\n".into(),
        "T 2.9 Track\n\nT 2.9.2 The track is wide.\n".into(),
    ]);
    assert_eq!(ids(&b), ["T 2.9", "T 2.9.2"]);
    assert_eq!(entry(&b, "T 2.9.2").text, "The track is wide.");
}

fn top(idx: &Index, q: &str) -> Vec<String> {
    idx.search(q, 10).into_iter().map(|h| h.id).collect()
}

#[test]
fn ranking_is_exact_id_then_title_then_body_then_partial() {
    let idx = Index::new(book());
    // exact id first, then the rules under it
    assert_eq!(top(&idx, "z 3.1"), ["Z 3.1", "Z 3.1.1", "Z 3.1.2"]);
    assert_eq!(top(&idx, "Z3.1.2")[0], "Z 3.1.2");
    // "beacon" is in the title of Z 3.1* and only in the body of Z 3.2.1
    let r = top(&idx, "beacon");
    let pos = |id: &str| {
        r.iter()
            .position(|x| x == id)
            .unwrap_or_else(|| panic!("{id} missing in {r:?}"))
    };
    assert!(
        pos("Z 3.1") < pos("Z 3.2.1") && pos("Z 3.1.1") < pos("Z 3.2.1"),
        "{r:?}"
    );
    // all words in the body beat a rule with only some of them
    let r = top(&idx, "wheel nuts");
    assert_eq!(r[0], "Z 3.2.1", "{r:?}");
    // a word seen in one place only, partially typed
    assert_eq!(top(&idx, "retent")[0], "Z 1.2.2");
    assert!(top(&idx, "zebra").is_empty());
}

#[test]
fn plurals_and_british_spelling_find_the_same_rule() {
    let idx = Index::new(build_rulebook(&[
        "T 2.7 Tires\n\nT 2.7.1 Wet tires need a minimum tread depth.\n".into(),
    ]));
    assert_eq!(top(&idx, "wet tyres")[0], "T 2.7.1");
    assert_eq!(top(&idx, "tread depths")[0], "T 2.7.1");
}

#[test]
fn snippets_are_short_and_highlight_the_matched_words() {
    let long = format!(
        "T 1.1 Heading\n\nT 1.1.1 Filler {} The magic number is 42 mm for the target gizmo. {}\n",
        "filler ".repeat(80),
        "tail ".repeat(80)
    );
    let idx = Index::new(build_rulebook(&[long]));
    let h = &idx.search("target gizmo", 3)[0];
    assert_eq!(h.id, "T 1.1.1");
    assert!(
        h.snippet.chars().count() <= 306,
        "{}",
        h.snippet.chars().count()
    );
    assert!(h.snippet.starts_with('…') && h.snippet.ends_with('…'));
    let units: Vec<u16> = h.snippet.encode_utf16().collect();
    let shown: Vec<String> = h
        .matches
        .iter()
        .map(|m| String::from_utf16(&units[m[0]..m[1]]).unwrap())
        .collect();
    assert_eq!(shown, ["target", "gizmo"]);
}

/// Real 2027 rulebook, local only: the lookups the quiz needs must land in the top 3.
#[test]
fn real_rulebook_lookups() {
    let Ok(path) = std::env::var("FSQ_RULES_PDF") else {
        eprintln!("FSQ_RULES_PDF not set: skipping the real-rulebook check");
        return;
    };
    let pages = extract_pages(Path::new(&path)).unwrap();
    let b = build_rulebook(&pages);
    eprintln!("{} pages, {} entries", b.pages, b.entries.len());
    let idx = Index::new(b);
    // (query, rule or family that must appear, how many top hits to look at)
    let cases = [
        ("skidpad", "D 4", 3),
        ("tread depth", "T 2.7.1", 3),
        ("wet tyres", "T 2.7.1", 5),
        // literally "track width" is the 3 m minimum width of the event tracks (D 6.1.1...), which rank first
        ("track width", "T 2.9.2", 20),
        ("vehicle track", "T 2.9.2", 3),
        ("insulation monitoring", "EV 6.3", 3),
        ("TSAL", "EV 4.10", 3),
        ("T 2.9.2", "T 2.9.2", 1),
    ];
    for (q, want, n) in cases {
        let r = idx.search(q, n);
        let got: Vec<String> = r
            .iter()
            .map(|h| format!("{} (p{})", h.id, h.page))
            .collect();
        eprintln!("{q:?} -> {got:?}");
        let family = format!("{want}.");
        assert!(
            r.iter().any(|h| h.id == want || h.id.starts_with(&family)),
            "{q:?} -> {got:?}, wanted {want}"
        );
    }
    let hit = &idx.search("track width", 1)[0];
    eprintln!("snippet: {}", hit.snippet);
}

#[test]
fn lowercase_definitions_are_rules_but_wrapped_cross_references_are_not() {
    // definition rows open in lowercase (T 5.1.3, D 1.1.6 in the real rulebook); a line that merely starts with a
    // reference ("T 4.3 and ...") continues the previous rule
    let b = build_rulebook(&[
        "T 5.1 Definitions\n\nT 5.1.3 upright position — a seat angled at 30° or less, as defined in\nT 4.3 and positioned per T 4.3.4.\n\nT 5.1.4 harness — the straps.\n"
            .into(),
    ]);
    assert_eq!(ids(&b), ["T 5.1", "T 5.1.3", "T 5.1.4"]);
    assert!(entry(&b, "T 5.1.3")
        .text
        .ends_with("T 4.3 and positioned per T 4.3.4."));
}
