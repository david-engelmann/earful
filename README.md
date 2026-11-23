# earful

Earful reads a folder of feedback and produces theme counts where every theme carries verbatim cited examples from the source items. A rerun on the same folder with the same taxonomy reproduces the output byte for byte. It is a local tool with no hosted service, no dashboard, and no network calls in its counting path.

## What it does

You point earful at a folder of app reviews, support mail saved as text, survey free text, or issue exports. It counts how many items support each theme in a taxonomy file you own and can edit. For every theme it prints the count, the share of analysed items, the item ids, and up to five quoted examples taken word for word from the source items. A verify command re-derives every count from the folder and fails if any quoted example is not found verbatim in its source item.

Earful does theme accounting, not theme discovery. It counts the themes its taxonomy names and reports plainly how many items matched no theme at all. Labels produced once by a person or a model can be frozen into a labelling file and counted from then on as data, so the counting itself never depends on a model call.

## How it is used

Version 1 reads three input shapes. A folder of .txt or .md files with one feedback item per file. A CSV file with a required column named text and optional columns named id, source, date, and rating. A JSONL file with a required field named text and the same optional fields. Other shapes, including mail archives, PDF files, and live connectors, are refused in version 1 and can be converted into one of the supported shapes first.

A run produces two files. The file themes.json is the machine readable record with the taxonomy version and hash, the corpus hash, per theme counts and item ids, and the citations. The file themes.md is the same result as a readable summary.

## Current state

The first two build slices are in. The counting core, the folder input shape, the verify command, and the audit command are implemented in Rust with the core crate on the standard library only, and the command line carries count, verify, audit, and score. The propose path is built as well, with propose, audit-labels, and freeze commands, a labelling file format the counter accepts only in its frozen state, and the precision gate enforced at freeze time. The design was tested before the repository was created by running a small prototype of the counting core over 600 real app reviews, and the shipped taxonomy 0.2 was measured on that same corpus under the precision gate described in docs/AUDIT.md. Every counted theme in taxonomy 0.2 scored precision 1.00 on its sampled citations after two themes were redesigned and two were pulled for sitting under the measurement floor. The full record is in docs/AUDIT-RECORD-0.2.md, with the design in docs/DESIGN.md and the v1 input fence in docs/AMENDMENT.md.

Still to come are the CSV and JSONL adapters. They are designed in the amendment and the slice plan, and they build on the counting core without changing its output format.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
