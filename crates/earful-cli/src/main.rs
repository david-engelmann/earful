//! The earful command line binary.
//!
//! Subcommands are count, verify, audit, and score. Argument
//! handling lives here. All counting behaviour lives in
//! earful-core.

use earful_core::{count, load_corpus, load_taxonomy, render_json, render_markdown};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some(cmd) = args.first() else {
        print_usage();
        return Ok(ExitCode::from(2));
    };
    match cmd.as_str() {
        "count" => cmd_count(&args[1..]),
        "verify" => cmd_verify(&args[1..]),
        "audit" => cmd_audit(&args[1..]),
        "score" => cmd_score(&args[1..]),
        _ => {
            print_usage();
            Ok(ExitCode::from(2))
        }
    }
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  earful count --folder DIR --taxonomy FILE --out DIR");
    eprintln!("  earful verify --folder DIR --taxonomy FILE --themes FILE");
    eprintln!("  earful audit --folder DIR --taxonomy FILE --out DIR");
    eprintln!("  earful score --sample FILE --verdicts FILE");
}

fn flag(args: &[String], name: &str) -> Result<String, String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == name {
            return iter
                .next()
                .cloned()
                .ok_or_else(|| format!("flag {name} needs a value"));
        }
    }
    Err(format!("missing required flag {name}"))
}

fn load(folder: &str, taxonomy: &str) -> Result<(earful_core::Corpus, earful_core::Taxonomy), String> {
    let corpus = load_corpus(Path::new(folder))?;
    let bytes = std::fs::read(taxonomy).map_err(|e| format!("cannot read taxonomy {taxonomy}: {e}"))?;
    let taxonomy = load_taxonomy(&bytes)?;
    Ok((corpus, taxonomy))
}

fn cmd_count(args: &[String]) -> Result<ExitCode, String> {
    let folder = flag(args, "--folder")?;
    let taxonomy_path = flag(args, "--taxonomy")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(&folder, &taxonomy_path)?;
    let report = count(&corpus, &taxonomy);
    let out_dir = PathBuf::from(&out);
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create out dir: {e}"))?;
    std::fs::write(out_dir.join("themes.json"), render_json(&report))
        .map_err(|e| format!("cannot write themes.json: {e}"))?;
    std::fs::write(out_dir.join("themes.md"), render_markdown(&report))
        .map_err(|e| format!("cannot write themes.md: {e}"))?;
    println!(
        "items={} untagged={} exact_duplicates={} near_duplicate_pairs={} themes={} suppressed={}",
        report.items_total,
        report.untagged_item_ids.len(),
        report.exact_duplicates.len(),
        report.near_duplicate_pairs.len(),
        report.themes.len(),
        report.suppressed.len()
    );
    for row in &report.themes {
        println!("  {}: {}", row.id, row.item_ids.len());
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_verify(args: &[String]) -> Result<ExitCode, String> {
    let folder = flag(args, "--folder")?;
    let taxonomy_path = flag(args, "--taxonomy")?;
    let themes_path = flag(args, "--themes")?;
    let (corpus, taxonomy) = load(&folder, &taxonomy_path)?;
    let themes_text = std::fs::read_to_string(&themes_path)
        .map_err(|e| format!("cannot read themes file {themes_path}: {e}"))?;
    let failures = earful_core::verify(&corpus, &taxonomy, &themes_text);
    if failures.is_empty() {
        println!("ALL CHECKS PASS");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("FAILURES {}", failures.len());
        for f in &failures {
            println!(" - {f}");
        }
        Ok(ExitCode::from(1))
    }
}

fn cmd_audit(args: &[String]) -> Result<ExitCode, String> {
    let folder = flag(args, "--folder")?;
    let taxonomy_path = flag(args, "--taxonomy")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(&folder, &taxonomy_path)?;
    let report = count(&corpus, &taxonomy);
    let entries = earful_core::sample(&report);
    let item_texts: std::collections::HashMap<String, String> = corpus
        .items
        .iter()
        .map(|i| (i.id.clone(), i.text.clone()))
        .collect();
    let out_dir = PathBuf::from(&out);
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create out dir: {e}"))?;
    std::fs::write(
        out_dir.join("audit.jsonl"),
        earful_core::render_sample_jsonl(&entries),
    )
    .map_err(|e| format!("cannot write audit.jsonl: {e}"))?;
    std::fs::write(
        out_dir.join("audit.md"),
        earful_core::render_worksheet(&entries, &item_texts),
    )
    .map_err(|e| format!("cannot write audit.md: {e}"))?;
    println!("sampled {} citations across {} themes", entries.len(), report.themes.len());
    Ok(ExitCode::SUCCESS)
}

fn cmd_score(args: &[String]) -> Result<ExitCode, String> {
    let sample_path = flag(args, "--sample")?;
    let verdicts_path = flag(args, "--verdicts")?;
    let sample_text = std::fs::read_to_string(&sample_path)
        .map_err(|e| format!("cannot read sample {sample_path}: {e}"))?;
    let verdicts_text = std::fs::read_to_string(&verdicts_path)
        .map_err(|e| format!("cannot read verdicts {verdicts_path}: {e}"))?;
    let scores = earful_core::score(&sample_text, &verdicts_text)?;
    let mut any_fail = false;
    for s in &scores {
        match (s.precision, s.passed) {
            (Some(p), Some(passed)) => {
                if !passed {
                    any_fail = true;
                }
                println!(
                    "{}: sampled {} supports {} precision {:.2} {}",
                    s.theme_id,
                    s.sampled,
                    s.supports,
                    p,
                    if passed { "PASS" } else { "FAIL" }
                );
            }
            _ => {
                println!(
                    "{}: sampled {} supports {} UNMEASURED ({} citations in total)",
                    s.theme_id, s.sampled, s.supports, s.theme_total
                );
            }
        }
    }
    Ok(if any_fail {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
