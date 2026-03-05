/// Core indexing logic: file scanning, chunking, embedding, and search.
use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::db::Db;
use crate::embed::{self, Provider};

const SUPPORTED_EXTENSIONS: &[&str] = &["md", "txt", "rst", "org", "adoc"];
const SNIPPET_LEN: usize = 300;

// ---------------------------------------------------------------------------
// Public command handlers
// ---------------------------------------------------------------------------

pub fn init() -> Result<()> {
    let db_path = db_path()?;
    if db_path.exists() {
        println!("engram: already initialized at {}", db_path.display());
        return Ok(());
    }

    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let provider = embed::detect_provider();
    let db = Db::open(&db_path)?;
    db.init(provider.dims(), provider.name())?;

    println!("✓ Initialized engram at {}", db_path.display());
    println!("  Provider: {}", provider.name());
    println!("  Dimensions: {}", provider.dims());
    Ok(())
}

pub fn add(paths: &[String], recursive: bool) -> Result<()> {
    let db_path = require_db()?;
    let db = Db::open(&db_path)?;
    let provider = load_provider(&db)?;

    let files = collect_files(paths, recursive);
    if files.is_empty() {
        println!("No supported files found.");
        return Ok(());
    }

    let bar = progress_bar(files.len() as u64, "Indexing");
    let mut added = 0usize;
    let mut skipped = 0usize;

    for path in &files {
        bar.set_message(path.display().to_string());

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let path_str = path.to_string_lossy().to_string();

        // Skip unchanged files
        if db.get_hash(&path_str)?.as_deref() == Some(&hash) {
            skipped += 1;
            bar.inc(1);
            continue;
        }

        let snippet = make_snippet(&content);
        let doc_id = db.upsert_document(&path_str, &hash, &snippet)?;
        let embedding = embed::embed(&content, &provider)
            .with_context(|| format!("Failed to embed {}", path.display()))?;
        db.insert_chunk(doc_id, &embedding)?;

        added += 1;
        bar.inc(1);
    }

    bar.finish_and_clear();
    println!("✓ Indexed {added} files ({skipped} unchanged, {} total)", db.document_count()?);
    Ok(())
}

pub fn search(query: &str, limit: usize, show_path: bool) -> Result<()> {
    let db_path = require_db()?;
    let db = Db::open(&db_path)?;
    let provider = load_provider(&db)?;

    let embedding = embed::embed(query, &provider)?;
    let results = db.search(&embedding, limit)?;

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    for (i, r) in results.iter().enumerate() {
        println!("{:2}. {} (score: {:.3})", i + 1, r.path, 1.0 - r.distance);
        if !show_path {
            println!("    {}\n", r.snippet.replace('\n', " "));
        }
    }
    Ok(())
}

pub fn remove(paths: &[String]) -> Result<()> {
    let db_path = require_db()?;
    let db = Db::open(&db_path)?;

    for path in paths {
        if db.remove_document(path)? {
            println!("✓ Removed {path}");
        } else {
            println!("  Not found: {path}");
        }
    }
    Ok(())
}

pub fn rebuild() -> Result<()> {
    let db_path = require_db()?;
    let db = Db::open(&db_path)?;
    let paths = db.all_paths()?;
    drop(db);

    // Delete and recreate the database
    std::fs::remove_file(&db_path)?;
    let provider = embed::detect_provider();
    let db = Db::open(&db_path)?;
    db.init(provider.dims(), provider.name())?;
    drop(db);

    let path_strings: Vec<String> = paths;
    add(&path_strings, false)
}

pub fn status() -> Result<()> {
    let db_path = require_db()?;
    let db = Db::open(&db_path)?;

    let count = db.document_count()?;
    let provider = db.get_meta("provider")?.unwrap_or_else(|| "unknown".into());
    let dims = db.get_meta("dims")?.unwrap_or_else(|| "?".into());
    let size = std::fs::metadata(&db_path)?.len();

    println!("engram index: {}", db_path.display());
    println!("  Documents : {count}");
    println!("  Provider  : {provider}");
    println!("  Dimensions: {dims}");
    println!("  Index size: {:.1} MB", size as f64 / 1_048_576.0);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn db_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    Ok(home.join(".engram").join("index.db"))
}

fn require_db() -> Result<PathBuf> {
    let path = db_path()?;
    if !path.exists() {
        bail!("engram not initialized. Run `engram init` first.");
    }
    Ok(path)
}

fn load_provider(db: &Db) -> Result<Provider> {
    let name = db.get_meta("provider")?.unwrap_or_default();
    let dims: usize = db.get_meta("dims")?.unwrap_or_default().parse().unwrap_or(1536);

    Ok(match name.as_str() {
        "openai/text-embedding-3-small" => Provider::OpenAiSmall,
        "ollama/nomic-embed-text" => {
            let base_url = std::env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            Provider::OllamaNomic { base_url }
        }
        _ => {
            // Try to auto-detect and warn if it differs
            let detected = embed::detect_provider();
            eprintln!("warning: unknown provider '{name}', using detected: {}", detected.name());
            detected
        }
    })
}

fn collect_files(paths: &[String], recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_file() {
            if is_supported(path) {
                files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            let walker = WalkDir::new(path).max_depth(if recursive { usize::MAX } else { 1 });
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && is_supported(entry.path()) {
                    files.push(entry.into_path());
                }
            }
        }
    }
    files
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e))
        .unwrap_or(false)
}

fn make_snippet(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.len() <= SNIPPET_LEN {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..SNIPPET_LEN])
    }
}

fn progress_bar(len: u64, msg: &str) -> ProgressBar {
    let bar = ProgressBar::new(len);
    bar.set_style(
        ProgressStyle::with_template("{msg:30} [{bar:40}] {pos}/{len}")
            .unwrap()
            .progress_chars("=> "),
    );
    bar.set_message(msg.to_string());
    bar
}
