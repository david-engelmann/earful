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
        "propose" => cmd_propose(&args[1..]),
        "audit-labels" => cmd_audit_labels(&args[1..]),
        "freeze" => cmd_freeze(&args[1..]),
        _ => {
            print_usage();
            Ok(ExitCode::from(2))
        }
    }
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("usage: every command takes --folder DIR or --input FILE, plus --taxonomy FILE");
    eprintln!("  earful count --folder DIR --taxonomy FILE --out DIR [--labels FILE]");
    eprintln!("  earful verify --folder DIR --taxonomy FILE --themes FILE [--labels FILE]");
    eprintln!("  earful audit --folder DIR --taxonomy FILE --out DIR");
    eprintln!("  earful score --sample FILE --verdicts FILE");
    eprintln!("  earful propose --folder DIR --taxonomy FILE --out DIR");
    eprintln!("  earful audit-labels --labels FILE --folder DIR --taxonomy FILE --out DIR");
    eprintln!(
        "  earful freeze --draft FILE --folder DIR --taxonomy FILE --verdicts FILE --reviewer NAME --out FILE"
    );
}

fn flag(args: &[String], name: &str) -> Result<String, String> {
    opt_flag(args, name).ok_or_else(|| format!("missing required flag {name}"))
}

fn opt_flag(args: &[String], name: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == name {
            return iter.next().cloned();
        }
    }
    None
}

fn load_labels(args: &[String]) -> Result<Option<(earful_core::LabellingFile, String)>, String> {
    match opt_flag(args, "--labels") {
        None => Ok(None),
        Some(path) => {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read labels {path}: {e}"))?;
            let file = earful_core::labels::parse_labelling(&text)?;
            Ok(Some((file, text)))
        }
    }
}

fn load(
    args: &[String],
    taxonomy: &str,
) -> Result<(earful_core::Corpus, earful_core::Taxonomy), String> {
    let corpus = match opt_flag(args, "--input") {
        Some(path) => earful_core::corpus::load_corpus_file(Path::new(&path))?,
        None => load_corpus(Path::new(&flag(args, "--folder")?))?,
    };
    let bytes =
        std::fs::read(taxonomy).map_err(|e| format!("cannot read taxonomy {taxonomy}: {e}"))?;
    let taxonomy = load_taxonomy(&bytes)?;
    Ok((corpus, taxonomy))
}

fn cmd_count(args: &[String]) -> Result<ExitCode, String> {
    let taxonomy_path = flag(args, "--taxonomy")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(args, &taxonomy_path)?;
    let labels = load_labels(args)?;
    let report = earful_core::count_with_labels(
        &corpus,
        &taxonomy,
        labels.as_ref().map(|(f, t)| (f, t.as_str())),
    )?;
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
    let taxonomy_path = flag(args, "--taxonomy")?;
    let themes_path = flag(args, "--themes")?;
    let (corpus, taxonomy) = load(args, &taxonomy_path)?;
    let themes_text = std::fs::read_to_string(&themes_path)
        .map_err(|e| format!("cannot read themes file {themes_path}: {e}"))?;
    let labels = load_labels(args)?;
    let failures = earful_core::verify(
        &corpus,
        &taxonomy,
        &themes_text,
        labels.as_ref().map(|(f, t)| (f, t.as_str())),
    );
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
    let taxonomy_path = flag(args, "--taxonomy")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(args, &taxonomy_path)?;
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
    println!(
        "sampled {} citations across {} themes",
        entries.len(),
        report.themes.len()
    );
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

fn cmd_propose(args: &[String]) -> Result<ExitCode, String> {
    let taxonomy_path = flag(args, "--taxonomy")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(args, &taxonomy_path)?;
    let report = count(&corpus, &taxonomy);
    let out_dir = PathBuf::from(&out);
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create out dir: {e}"))?;
    let sheet = earful_core::propose::worksheet(&report, &corpus, &taxonomy);
    std::fs::write(out_dir.join("worksheet.jsonl"), sheet)
        .map_err(|e| format!("cannot write worksheet.jsonl: {e}"))?;
    println!(
        "worksheet holds {} untagged items",
        report.untagged_item_ids.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn cmd_audit_labels(args: &[String]) -> Result<ExitCode, String> {
    let labels_path = flag(args, "--labels")?;
    let taxonomy_path = flag(args, "--taxonomy")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(args, &taxonomy_path)?;
    let text = std::fs::read_to_string(&labels_path)
        .map_err(|e| format!("cannot read labels {labels_path}: {e}"))?;
    let file = earful_core::labels::parse_labelling(&text)?;
    let failures = earful_core::labels::validate(&file.labels, &corpus, &taxonomy);
    if !failures.is_empty() {
        for f in &failures {
            eprintln!(" - {f}");
        }
        return Err(format!(
            "labels fail validation with {} problems",
            failures.len()
        ));
    }
    let pools = earful_core::propose::label_pools(&file, &taxonomy);
    let entries = earful_core::audit::sample_pools(&pools);
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
    println!(
        "sampled {} labels across {} themes",
        entries.len(),
        pools.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn cmd_freeze(args: &[String]) -> Result<ExitCode, String> {
    let draft_path = flag(args, "--draft")?;
    let taxonomy_path = flag(args, "--taxonomy")?;
    let verdicts_path = flag(args, "--verdicts")?;
    let reviewer = flag(args, "--reviewer")?;
    let out = flag(args, "--out")?;
    let (corpus, taxonomy) = load(args, &taxonomy_path)?;
    let draft_text = std::fs::read_to_string(&draft_path)
        .map_err(|e| format!("cannot read draft {draft_path}: {e}"))?;
    let draft = earful_core::labels::parse_labelling(&draft_text)?;
    let verdicts_text = std::fs::read_to_string(&verdicts_path)
        .map_err(|e| format!("cannot read verdicts {verdicts_path}: {e}"))?;
    // The freeze is a human act. The measurement runs first,
    // inside earful_core::propose::freeze, and only a passing
    // draft reaches this prompt. There is no flag that skips
    // the confirmation.
    let frozen =
        earful_core::propose::freeze(&draft, &corpus, &taxonomy, &verdicts_text, &reviewer)?;
    println!(
        "The draft passed the precision gate. Freezing {} labels as reviewer {}.",
        draft.labels.len(),
        reviewer
    );
    print!("Type freeze to confirm: ");
    use std::io::Write as _;
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("cannot read confirmation: {e}"))?;
    if line.trim() != "freeze" {
        eprintln!("freeze aborted, nothing written");
        return Ok(ExitCode::from(1));
    }
    std::fs::write(&out, frozen).map_err(|e| format!("cannot write frozen file {out}: {e}"))?;
    println!("froze {} labels to {out}", draft.labels.len());
    Ok(ExitCode::SUCCESS)
}
