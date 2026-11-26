//! End to end tests over a fixture corpus the test builds
//! itself. They pin the two properties the product stands on.
//! A rerun reproduces the output byte for byte, and verify
//! accepts a genuine report and rejects a tampered one.

use std::path::PathBuf;
use std::process::Command;

const TAXONOMY: &str = r#"{"version": "test-1", "themes": [
    {"id": "crashes", "label": "Crashes", "kind": "operational_issue",
     "phrases": ["crash", "crashes"]},
    {"id": "ads", "label": "Ads", "kind": "operational_issue",
     "phrases": ["ads"]}
]}"#;

fn fixture(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("earful-cli-test-{}-{}", std::process::id(), tag));
    let corpus = base.join("corpus");
    std::fs::create_dir_all(&corpus).unwrap();
    let items = [
        ("item-01", "It crashes every single day."),
        ("item-02", "Another crash this morning."),
        ("item-03", "The app crashes at noon daily."),
        ("item-04", "Too many ads in this thing."),
        ("item-05", "Ads everywhere I look now."),
        ("item-06", "The ads are unbearable here."),
        ("item-07", "He reads threads on his headset."),
        ("item-08", "It crashes every single day."),
    ];
    for (id, text) in items {
        std::fs::write(corpus.join(format!("{id}.txt")), text).unwrap();
    }
    std::fs::write(base.join("taxonomy.json"), TAXONOMY).unwrap();
    base
}

fn earful(base: &PathBuf, args: &[&str]) -> std::process::Output {
    earful_stdin(base, args, None)
}

fn earful_stdin(base: &PathBuf, args: &[&str], stdin_text: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_earful"));
    cmd.args(args).current_dir(base);
    if stdin_text.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(text) = stdin_text {
        use std::io::Write as _;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

#[test]
fn count_rerun_is_byte_identical_and_verify_passes() {
    let base = fixture("rerun");
    let corpus = base.join("corpus").to_string_lossy().to_string();
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();

    let out1 = base.join("out1").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out1,
        ],
    );
    assert!(
        run.status.success(),
        "count failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let json1 = std::fs::read(base.join("out1/themes.json")).unwrap();
    let md1 = std::fs::read(base.join("out1/themes.md")).unwrap();

    let out2 = base.join("out2").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out2,
        ],
    );
    assert!(run.status.success());
    assert_eq!(json1, std::fs::read(base.join("out2/themes.json")).unwrap());
    assert_eq!(md1, std::fs::read(base.join("out2/themes.md")).unwrap());

    let text = String::from_utf8(json1).unwrap();
    assert!(
        text.contains("\"items_total\": 7"),
        "duplicate not excluded: {text}"
    );
    assert!(
        text.contains("\"crashes\""),
        "crashes theme missing: {text}"
    );

    let themes = base.join("out1/themes.json").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "verify",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--themes",
            &themes,
        ],
    );
    assert!(
        run.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&run.stdout)
    );

    // Tamper with a count and verify must reject the file.
    let tampered = text.replace("\"count\": 3", "\"count\": 4");
    assert_ne!(tampered, text);
    std::fs::write(base.join("tampered.json"), &tampered).unwrap();
    let tampered_path = base.join("tampered.json").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "verify",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--themes",
            &tampered_path,
        ],
    );
    assert!(!run.status.success(), "verify accepted a tampered file");
}

#[test]
fn getting_started_example_matches_its_readme() {
    // The example under examples/getting-started promises
    // exact numbers in its README. This test runs it, so the
    // promise cannot drift from what the tool produces.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = root.join("examples/getting-started");
    let corpus = example.join("corpus").to_string_lossy().to_string();
    let tax = example.join("taxonomy.json").to_string_lossy().to_string();
    let base = fixture("example");
    let out = base.join("example-out").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out,
        ],
    );
    assert!(
        run.status.success(),
        "example count failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text =
        String::from_utf8(std::fs::read(base.join("example-out/themes.json")).unwrap()).unwrap();
    assert!(
        text.contains("\"items_total\": 11"),
        "example total: {text}"
    );
    assert!(
        text.contains("\"items_untagged\": 1"),
        "example untagged: {text}"
    );
    for id in ["crashes", "praise", "wishlist"] {
        assert!(text.contains(&format!("\"id\": \"{id}\"")), "missing {id}");
    }
    assert_eq!(
        text.matches("\"count\": 3").count(),
        3,
        "theme counts: {text}"
    );
    assert!(
        text.contains("\"battery\"") && text.contains("\"suppressed_themes\""),
        "battery not suppressed: {text}"
    );
    let themes = base
        .join("example-out/themes.json")
        .to_string_lossy()
        .to_string();
    let run = earful(
        &base,
        &[
            "verify",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--themes",
            &themes,
        ],
    );
    assert!(
        run.status.success(),
        "example verify failed: {}",
        String::from_utf8_lossy(&run.stdout)
    );
}

#[test]
fn labelling_example_flow_holds() {
    // The shipped labelling example, run end to end. The
    // README promises billing at five from the labels and
    // crashes at three from the lexicon, with nothing left
    // untagged. If the tool changes, this test makes the
    // example change with it.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = root
        .join("examples/labelling/corpus")
        .to_string_lossy()
        .to_string();
    let tax = root
        .join("examples/labelling/taxonomy.json")
        .to_string_lossy()
        .to_string();
    let base = fixture("labex");
    let sheet = base.join("sheet").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "propose",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &sheet,
        ],
    );
    assert!(run.status.success(), "propose failed");
    let worksheet = std::fs::read_to_string(base.join("sheet/worksheet.jsonl")).unwrap();
    assert_eq!(worksheet.lines().count(), 6, "header plus five untagged");

    let draft = root
        .join("examples/labelling/draft.jsonl")
        .to_string_lossy()
        .to_string();
    let audit_dir = base.join("laudit").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "audit-labels",
            "--labels",
            &draft,
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &audit_dir,
        ],
    );
    assert!(run.status.success(), "audit-labels failed");
    let sample = std::fs::read_to_string(base.join("laudit/audit.jsonl")).unwrap();
    assert_eq!(sample.lines().count(), 5);
    let verdicts = sample.replace("\"verdict\":\"\"", "\"verdict\":\"support\"");
    assert_ne!(verdicts, sample);
    std::fs::write(base.join("verdicts.jsonl"), &verdicts).unwrap();
    let verdicts_path = base.join("verdicts.jsonl").to_string_lossy().to_string();
    let frozen = base.join("frozen.jsonl").to_string_lossy().to_string();
    let run = earful_stdin(
        &base,
        &[
            "freeze",
            "--draft",
            &draft,
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--verdicts",
            &verdicts_path,
            "--reviewer",
            "Example Reviewer",
            "--out",
            &frozen,
        ],
        Some("freeze\n"),
    );
    assert!(
        run.status.success(),
        "freeze failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let out = base.join("out").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--labels",
            &frozen,
            "--out",
            &out,
        ],
    );
    assert!(run.status.success(), "labelled count failed");
    let md = std::fs::read_to_string(base.join("out/themes.md")).unwrap();
    assert!(md.contains("Billing problems (5 items"), "md: {md}");
    assert!(md.contains("Crashes (3 items"), "md: {md}");
    assert!(md.contains("Items with no theme 0."), "md: {md}");
    let json = std::fs::read_to_string(base.join("out/themes.json")).unwrap();
    assert!(json.contains("\"labelling_sha256\": \""), "json: {json}");
}

#[test]
fn import_example_flow_holds() {
    // The shipped export example. The README promises the
    // reconciliation numbers, nine lines, nine read, eight
    // analysed, one duplicate, and crashes counted at three.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let export = root
        .join("examples/import/export.jsonl")
        .to_string_lossy()
        .to_string();
    let tax = root
        .join("examples/import/taxonomy.json")
        .to_string_lossy()
        .to_string();
    let base = fixture("impex");
    let run = earful(&base, &["check-export", "--input", &export]);
    assert!(run.status.success(), "check-export failed");
    let out = base.join("out").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "import",
            "--input",
            &export,
            "--taxonomy",
            &tax,
            "--out",
            &out,
        ],
    );
    assert!(run.status.success(), "import failed");
    let stdout = String::from_utf8(run.stdout).unwrap();
    for needle in [
        "export lines: 9",
        "items read: 9",
        "items analysed: 8 (exact duplicates 1)",
        "decode warnings: 0",
        "reconciliation: PASS",
    ] {
        assert!(stdout.contains(needle), "missing {needle}: {stdout}");
    }
    let md = std::fs::read_to_string(base.join("out/themes.md")).unwrap();
    assert!(md.contains("Crashes (3 items"), "md: {md}");
    assert!(md.contains("Items with no theme 1."), "md: {md}");
}

#[test]
fn verify_rejects_a_tampered_markdown() {
    let base = fixture("mdtamper");
    let corpus = base.join("corpus").to_string_lossy().to_string();
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();
    let out = base.join("out").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out,
        ],
    );
    assert!(run.status.success());
    // Edit a count in the Markdown only. The JSON still
    // verifies on its own terms, so only the rendering check
    // can catch this.
    let md_path = base.join("out/themes.md");
    let md = std::fs::read_to_string(&md_path).unwrap();
    let edited = md.replacen("(3 items", "(9 items", 1);
    assert_ne!(edited, md, "fixture markdown has no editable count");
    std::fs::write(&md_path, edited).unwrap();
    let themes = base.join("out/themes.json").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "verify",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--themes",
            &themes,
        ],
    );
    assert!(!run.status.success(), "verify accepted a tampered markdown");
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("themes.md differs"),
        "unexpected verify output: {}",
        String::from_utf8_lossy(&run.stdout)
    );
}

#[test]
fn audit_sample_and_score_roundtrip() {
    let base = fixture("audit");
    let corpus = base.join("corpus").to_string_lossy().to_string();
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();
    let out = base.join("audit").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "audit",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out,
        ],
    );
    assert!(
        run.status.success(),
        "audit failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let sample = std::fs::read_to_string(base.join("audit/audit.jsonl")).unwrap();
    assert_eq!(sample.lines().count(), 6);
    // Both themes carry only 3 citations, so both score as
    // unmeasured even with full support verdicts.
    let verdicts = sample.replace("\"verdict\":\"\"", "\"verdict\":\"support\"");
    assert_ne!(verdicts, sample);
    std::fs::write(base.join("verdicts.jsonl"), &verdicts).unwrap();
    let sample_path = base.join("audit/audit.jsonl").to_string_lossy().to_string();
    let verdicts_path = base.join("verdicts.jsonl").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "score",
            "--sample",
            &sample_path,
            "--verdicts",
            &verdicts_path,
        ],
    );
    assert!(run.status.success());
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert!(
        stdout.contains("UNMEASURED"),
        "expected unmeasured themes: {stdout}"
    );
}

#[test]
fn propose_freeze_count_flow() {
    let base = fixture("propose");
    let corpus = base.join("corpus").to_string_lossy().to_string();
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();

    // Propose writes a worksheet holding the one untagged item.
    let sheet_dir = base.join("sheet").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "propose",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &sheet_dir,
        ],
    );
    assert!(
        run.status.success(),
        "propose failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let sheet = std::fs::read_to_string(base.join("sheet/worksheet.jsonl")).unwrap();
    assert_eq!(sheet.lines().count(), 2);
    assert!(sheet.contains("item-07"));

    // A person labels item-07 as crashes, outside the tool.
    let draft = "{\"item_id\":\"item-07\",\"theme_id\":\"crashes\",\"span\":\"He reads threads on his headset.\",\"labeler\":\"test\",\"status\":\"draft\"}\n";
    std::fs::write(base.join("draft.jsonl"), draft).unwrap();
    let draft_path = base.join("draft.jsonl").to_string_lossy().to_string();

    // The counter refuses the draft outright.
    let out_draft = base.join("out-draft").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out_draft,
            "--labels",
            &draft_path,
        ],
    );
    assert!(!run.status.success(), "counter accepted a draft file");

    // Audit the draft labels and score the sample as support.
    let audit_dir = base.join("label-audit").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "audit-labels",
            "--labels",
            &draft_path,
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &audit_dir,
        ],
    );
    assert!(
        run.status.success(),
        "audit-labels failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let sample = std::fs::read_to_string(base.join("label-audit/audit.jsonl")).unwrap();
    assert_eq!(sample.lines().count(), 1);
    let verdicts = sample.replace("\"verdict\":\"\"", "\"verdict\":\"support\"");
    std::fs::write(base.join("verdicts.jsonl"), &verdicts).unwrap();
    let verdicts_path = base.join("verdicts.jsonl").to_string_lossy().to_string();
    let frozen_path = base.join("frozen.jsonl").to_string_lossy().to_string();

    // Freeze with the human confirmation typed at the prompt.
    let run = earful_stdin(
        &base,
        &[
            "freeze",
            "--draft",
            &draft_path,
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--verdicts",
            &verdicts_path,
            "--reviewer",
            "test-reviewer",
            "--out",
            &frozen_path,
        ],
        Some("freeze\n"),
    );
    assert!(
        run.status.success(),
        "freeze failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let frozen = std::fs::read_to_string(base.join("frozen.jsonl")).unwrap();
    assert!(frozen.contains("earful-labelling-header"));

    // Count from the frozen file. Crashes rises to four items
    // and nothing stays untagged.
    let out = base.join("out-labelled").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--out",
            &out,
            "--labels",
            &frozen_path,
        ],
    );
    assert!(
        run.status.success(),
        "labelled count failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = std::fs::read_to_string(base.join("out-labelled/themes.json")).unwrap();
    assert!(text.contains("\"count\": 4"), "label not counted: {text}");
    assert!(!text.contains("\"labelling_sha256\": null"));

    // Verify accepts the labelled report with the same file.
    let themes = base
        .join("out-labelled/themes.json")
        .to_string_lossy()
        .to_string();
    let run = earful(
        &base,
        &[
            "verify",
            "--folder",
            &corpus,
            "--taxonomy",
            &tax,
            "--themes",
            &themes,
            "--labels",
            &frozen_path,
        ],
    );
    assert!(
        run.status.success(),
        "verify of labelled run failed: {}",
        String::from_utf8_lossy(&run.stdout)
    );
}

#[test]
fn jsonl_input_counts_like_a_folder() {
    let base = fixture("jsonl");
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();
    let jsonl = "{\"id\": \"j1\", \"text\": \"It crashes every day.\"}\n\
                 {\"id\": \"j2\", \"text\": \"Another crash here.\"}\n\
                 {\"id\": \"j3\", \"text\": \"The app crashes at noon.\"}\n\
                 {\"id\": \"j4\", \"text\": \"He reads threads on his headset.\"}\n";
    std::fs::write(base.join("input.jsonl"), jsonl).unwrap();
    let input = base.join("input.jsonl").to_string_lossy().to_string();
    let out = base.join("out-jsonl").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "count",
            "--input",
            &input,
            "--taxonomy",
            &tax,
            "--out",
            &out,
        ],
    );
    assert!(
        run.status.success(),
        "jsonl count failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = std::fs::read_to_string(base.join("out-jsonl/themes.json")).unwrap();
    assert!(text.contains("\"items_total\": 4"), "wrong total: {text}");
    assert!(text.contains("\"crashes\""), "crashes missing: {text}");
    assert!(text.contains("\"j4\""), "untagged id missing: {text}");
}

#[test]
fn check_export_accepts_the_spec_examples_and_refuses_a_bad_file() {
    let base = fixture("export");
    // The two synthetic example lines from docs/DUSKFM-EXPORT.md.
    let good = "{\"id\": \"fb-0001\", \"source\": \"in_app\", \"date\": \"2026-11-01T14:23:00Z\", \"text\": \"The player stops playing when the screen locks and I have to reopen the app.\"}\n{\"id\": \"fb-0002\", \"source\": \"support\", \"date\": \"2026-11-02T09:05:00Z\", \"rating\": 2, \"text\": \"Sync between my phone and desktop has not worked since the last update.\"}\n";
    std::fs::write(base.join("duskfm-feedback.jsonl"), good).unwrap();
    let input = base
        .join("duskfm-feedback.jsonl")
        .to_string_lossy()
        .to_string();
    let run = earful(&base, &["check-export", "--input", &input]);
    assert!(
        run.status.success(),
        "spec examples refused: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let bad = "{\"id\": \"fb-0002\", \"source\": \"chat\", \"text\": \"x\"}\n";
    std::fs::write(base.join("bad.jsonl"), bad).unwrap();
    let bad_path = base.join("bad.jsonl").to_string_lossy().to_string();
    let run = earful(&base, &["check-export", "--input", &bad_path]);
    assert!(!run.status.success(), "bad export accepted");
}

#[test]
fn import_gates_on_the_contract_and_reconciles() {
    let base = fixture("import");
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();
    // The spec's two synthetic lines, as an export file.
    let good = "{\"id\": \"fb-0001\", \"source\": \"in_app\", \"date\": \"2026-11-01T14:23:00Z\", \"text\": \"The player stops playing when the screen locks and I have to reopen the app.\"}\n{\"id\": \"fb-0002\", \"source\": \"support\", \"date\": \"2026-11-02T09:05:00Z\", \"rating\": 2, \"text\": \"Sync between my phone and desktop has not worked since the last update.\"}\n";
    std::fs::write(base.join("duskfm-feedback.jsonl"), good).unwrap();
    let input = base
        .join("duskfm-feedback.jsonl")
        .to_string_lossy()
        .to_string();
    let out = base.join("imported").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "import",
            "--input",
            &input,
            "--taxonomy",
            &tax,
            "--out",
            &out,
        ],
    );
    assert!(
        run.status.success(),
        "import failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert!(stdout.contains("reconciliation: PASS"), "stdout: {stdout}");
    let text = std::fs::read_to_string(base.join("imported/themes.json")).unwrap();
    assert!(text.contains("\"items_total\": 2"), "wrong total: {text}");

    // An export outside the contract is refused before counting.
    let bad = "{\"id\": \"fb-0001\", \"source\": \"chat\", \"text\": \"x\"}\n";
    std::fs::write(base.join("bad.jsonl"), bad).unwrap();
    let bad_path = base.join("bad.jsonl").to_string_lossy().to_string();
    let out2 = base.join("imported-bad").to_string_lossy().to_string();
    let run = earful(
        &base,
        &[
            "import",
            "--input",
            &bad_path,
            "--taxonomy",
            &tax,
            "--out",
            &out2,
        ],
    );
    assert!(!run.status.success(), "import accepted a bad export");
    assert!(!base.join("imported-bad/themes.json").exists());
}
