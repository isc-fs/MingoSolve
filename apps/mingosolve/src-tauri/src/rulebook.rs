//! Rulebook commands: the user loads the official rules PDF once, the extracted rule entries are saved per year in
//! the app data dir (nothing is bundled or written to the repo), and searches run against that saved index.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::blocking;
use crate::rules_text::{build_rulebook, extract_pages, Hit, Index, Rulebook};

/// Fewer entries than this means the PDF is not a Formula Student rulebook (or its text could not be read).
const MIN_ENTRIES: usize = 50;

#[derive(Serialize, Deserialize)]
struct Saved {
    year: String,
    source: String,
    loaded_at: u64,
    book: Rulebook,
}

#[derive(Serialize)]
pub struct RulebookStatus {
    year: String,
    source: String,
    loaded_at: u64,
    pages: u32,
    entries: usize,
}

#[derive(Serialize)]
pub struct RuleHit {
    year: String,
    #[serde(flatten)]
    hit: Hit,
}

fn valid_year(year: &str) -> Result<(), String> {
    if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(format!("\"{year}\" is not a rules year like 2027"))
    }
}

/// Where the saved rulebooks live (`<app data dir>/rulebooks`), managed by the app at startup.
pub struct RulebookDir(pub PathBuf);

fn file(dir: &Path, year: &str) -> PathBuf {
    dir.join(format!("{year}.json"))
}

fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<Index>>> {
    static C: OnceLock<Mutex<HashMap<PathBuf, Arc<Index>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

fn read_saved(path: &Path) -> Result<Saved, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("could not read the saved rulebook: {e}"))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("the saved rulebook is damaged, load the PDF again ({e})"))
}

fn index(path: &Path) -> Result<Option<Arc<Index>>, String> {
    if let Some(i) = cache().lock().unwrap().get(path) {
        return Ok(Some(i.clone()));
    }
    if !path.exists() {
        return Ok(None);
    }
    let i = Arc::new(Index::new(read_saved(path)?.book));
    cache()
        .lock()
        .unwrap()
        .insert(path.to_path_buf(), i.clone());
    Ok(Some(i))
}

fn load(pdf: &Path, year: &str, dir: &Path) -> Result<RulebookStatus, String> {
    let pages = extract_pages(pdf)?;
    let book = build_rulebook(&pages);
    if book.entries.len() < MIN_ENTRIES {
        return Err(format!(
            "found only {} rules in this PDF: is it the official FS-Rules document?",
            book.entries.len()
        ));
    }
    let saved = Saved {
        year: year.to_string(),
        source: pdf
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        loaded_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        book,
    };
    std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let path = file(dir, year);
    let json = serde_json::to_vec(&saved).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("could not save the rulebook: {e}"))?;
    cache().lock().unwrap().remove(&path);
    Ok(status_of(&saved))
}

fn status_of(s: &Saved) -> RulebookStatus {
    RulebookStatus {
        year: s.year.clone(),
        source: s.source.clone(),
        loaded_at: s.loaded_at,
        pages: s.book.pages,
        entries: s.book.entries.len(),
    }
}

/// Reads the PDF at `path`, extracts and indexes it, and saves it as the rulebook of `year`.
#[tauri::command]
pub async fn load_rulebook(
    dir: State<'_, RulebookDir>,
    path: String,
    year: String,
) -> Result<RulebookStatus, String> {
    valid_year(&year)?;
    let dir = dir.0.clone();
    blocking(move || load(Path::new(&path), &year, &dir)).await
}

/// The rulebooks loaded so far, oldest year first.
#[tauri::command]
pub async fn rulebook_status(dir: State<'_, RulebookDir>) -> Result<Vec<RulebookStatus>, String> {
    let dir = dir.0.clone();
    blocking(move || {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            return Ok(vec![]);
        };
        let mut out: Vec<RulebookStatus> = rd
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| read_saved(&e.path()).ok())
            .map(|s| status_of(&s))
            .collect();
        out.sort_by(|a, b| a.year.cmp(&b.year));
        Ok(out)
    })
    .await
}

#[tauri::command]
pub async fn search_rules(
    dir: State<'_, RulebookDir>,
    query: String,
    year: String,
    limit: usize,
) -> Result<Vec<RuleHit>, String> {
    valid_year(&year)?;
    let path = file(&dir.0, &year);
    blocking(move || {
        let Some(i) = index(&path)? else {
            return Err(format!("no rulebook loaded for {year}"));
        };
        Ok(i.search(&query, limit)
            .into_iter()
            .map(|hit| RuleHit {
                year: year.clone(),
                hit,
            })
            .collect())
    })
    .await
}

#[tauri::command]
pub async fn remove_rulebook(dir: State<'_, RulebookDir>, year: String) -> Result<(), String> {
    valid_year(&year)?;
    let path = file(&dir.0, &year);
    blocking(move || {
        cache().lock().unwrap().remove(&path);
        match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("could not remove the rulebook: {e}"))
            }
            _ => Ok(()),
        }
    })
    .await
}
