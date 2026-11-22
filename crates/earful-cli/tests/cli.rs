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

fn fixture() -> PathBuf {
    let base = std::env::temp_dir().join(format!("earful-cli-test-{}", std::process::id()));
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
    Command::new(env!("CARGO_BIN_EXE_earful"))
        .current_dir(base)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn count_rerun_is_byte_identical_and_verify_passes() {
    let base = fixture();
    let corpus = base.join("corpus").to_string_lossy().to_string();
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();

    let out1 = base.join("out1").to_string_lossy().to_string();
    let run = earful(&base, &["count", "--folder", &corpus, "--taxonomy", &tax, "--out", &out1]);
    assert!(run.status.success(), "count failed: {}", String::from_utf8_lossy(&run.stderr));
    let json1 = std::fs::read(base.join("out1/themes.json")).unwrap();
    let md1 = std::fs::read(base.join("out1/themes.md")).unwrap();

    let out2 = base.join("out2").to_string_lossy().to_string();
    let run = earful(&base, &["count", "--folder", &corpus, "--taxonomy", &tax, "--out", &out2]);
    assert!(run.status.success());
    assert_eq!(json1, std::fs::read(base.join("out2/themes.json")).unwrap());
    assert_eq!(md1, std::fs::read(base.join("out2/themes.md")).unwrap());

    let text = String::from_utf8(json1).unwrap();
    assert!(text.contains("\"items_total\": 7"), "duplicate not excluded: {text}");
    assert!(text.contains("\"crashes\""), "crashes theme missing: {text}");

    let themes = base.join("out1/themes.json").to_string_lossy().to_string();
    let run = earful(&base, &["verify", "--folder", &corpus, "--taxonomy", &tax, "--themes", &themes]);
    assert!(run.status.success(), "verify failed: {}", String::from_utf8_lossy(&run.stdout));

    // Tamper with a count and verify must reject the file.
    let tampered = text.replace("\"count\": 3", "\"count\": 4");
    assert_ne!(tampered, text);
    std::fs::write(base.join("tampered.json"), &tampered).unwrap();
    let tampered_path = base.join("tampered.json").to_string_lossy().to_string();
    let run = earful(&base, &["verify", "--folder", &corpus, "--taxonomy", &tax, "--themes", &tampered_path]);
    assert!(!run.status.success(), "verify accepted a tampered file");
}

#[test]
fn audit_sample_and_score_roundtrip() {
    let base = fixture();
    let corpus = base.join("corpus").to_string_lossy().to_string();
    let tax = base.join("taxonomy.json").to_string_lossy().to_string();
    let out = base.join("audit").to_string_lossy().to_string();
    let run = earful(&base, &["audit", "--folder", &corpus, "--taxonomy", &tax, "--out", &out]);
    assert!(run.status.success(), "audit failed: {}", String::from_utf8_lossy(&run.stderr));
    let sample = std::fs::read_to_string(base.join("audit/audit.jsonl")).unwrap();
    assert_eq!(sample.lines().count(), 6);
    // Both themes carry only 3 citations, so both score as
    // unmeasured even with full support verdicts.
    let verdicts = sample.replace("\"verdict\":\"\"", "\"verdict\":\"support\"");
    assert_ne!(verdicts, sample);
    std::fs::write(base.join("verdicts.jsonl"), &verdicts).unwrap();
    let sample_path = base.join("audit/audit.jsonl").to_string_lossy().to_string();
    let verdicts_path = base.join("verdicts.jsonl").to_string_lossy().to_string();
    let run = earful(&base, &["score", "--sample", &sample_path, "--verdicts", &verdicts_path]);
    assert!(run.status.success());
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert!(stdout.contains("UNMEASURED"), "expected unmeasured themes: {stdout}");
}
