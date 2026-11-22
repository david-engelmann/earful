# The propose subcommand, design

Status is design, written 2026-11-21 under Main's press order. The gate ruling stands behind every line here. Propose is a version 1 component that drafts candidate labels for human freezing only. The counting core never calls a model and never consumes an unfrozen labelling file. If the precision measurement fails, propose comes out and the README says known themes only.

## What propose is for

The Phase 1 test left 48.7 percent of items untagged under a starter lexicon. Some of those items carry real themes the lexicon phrases miss, through inflection, paraphrase, or themes nobody wrote phrases for yet. Propose exists to close that gap once per corpus, under human control, without giving up the rerun property. A person or a model reads items and drafts labels. A person reviews the draft through the audit measurement. A person freezes the result. From the freeze onward the labels are data, and counting them is as deterministic as counting lexicon hits.

## The labelling file is the whole interface

One format serves the human labeller, the model labeller, the draft state, and the frozen state. The file is JSONL, one object per label, with these fields.

The field item_id names the item being labelled and must match an item in the corpus. The field theme_id names a theme in the taxonomy and must match a theme id there. The field span is a verbatim quote from the item that supports the label. The field labeler names who or what produced the label, as free text that must be present, for example a person's initials or a model name with its version. The field status is either draft or frozen. A frozen file also carries a header line, itself a JSON object, naming the corpus hash it was frozen against, the taxonomy hash, the reviewer who froze it, and the audit precision recorded at freeze time.

Validation is mechanical and runs on every load. Every item_id must exist in the corpus. Every theme_id must exist in the taxonomy. Every span must occur verbatim in its item, checked exactly as the verify subcommand checks citations. A file that fails any check is rejected whole, with the failing line numbers printed. The counting core accepts a labelling file only when its status is frozen and its header corpus hash and taxonomy hash match the run. A draft file passed to the counter is a hard error, never a silent ignore, because a silent ignore is how an unfrozen label set would leak into a published count.

## How propose works

Propose never embeds a model client and holds no API key. That is a deliberate boundary. A model client inside the tool would drag provider drift, key custody, and per run cost into a product whose claim is that counting is free, local, and reproducible. Instead propose works through files, in three steps any labeller can complete.

Step one is the worksheet. Running propose with a folder and a taxonomy writes worksheet.jsonl. Each line carries one untagged item in full, or one item per theme gap when the run is scoped, plus the taxonomy themes with their labels and plain language definitions. The worksheet order is sorted item id order, so the worksheet itself is reproducible.

Step two is labelling, outside the tool. A person fills in theme_id and span per line in an editor. Or the user hands worksheet.jsonl to their own chat tool with the instruction file propose carries, and the model returns completed lines. Either way the output is draft.jsonl in the labelling format with status draft on every line and the labeler field naming the person or the model and version. Earful does not care which labeller was used, and the record keeps the name so a reader can weigh it.

Step three is freeze, a separate subcommand. Freeze first runs the audit measurement described in docs/AUDIT.md against the draft, sampling its labels per theme. Freeze refuses when any theme with at least five sampled labels scores under four in five, and prints the failing themes and their scores. When the measurement passes, a human reviewer confirms at the prompt, and freeze writes the frozen labelling file with its header. The confirmation is a human act by design. No flag skips it, because the freeze is the moment non deterministic judgement becomes data the product will stand behind.

## What propose deliberately does not do

It does not label during a count. It does not update labels incrementally in version 1, so a corpus change means a new worksheet for the changed and new items and a new freeze, with the old frozen file kept beside the new one. It does not propose new themes in version 1. A labeller who finds a theme the taxonomy lacks writes it in a notes field on the worksheet line, and theme creation stays a taxonomy edit by a person, versioned like any other taxonomy change. It does not rank or summarise. Its whole output is labels with receipts.

## Build notes

Propose lands as slice 4, after the lexicon counter, the CSV and JSONL adapters, and the frozen labelling path in the core. The labelling format and its validation land earlier, in slice 3, because the core must consume frozen files before propose can draft them. The worksheet writer and the freeze subcommand are the only new code propose itself needs, and both are deterministic file transforms plus the audit call and the human confirmation.
