# The audit subcommand, design

Status is design, written 2026-11-21 under Main's press order. Audit is the measurement half of the precision gate that Main approved at the Phase 1 gate. The default taxonomy ships under this contract, and the propose freeze depends on it. A theme that fails the gate ships redesigned or not at all.

## What audit measures

Audit measures one thing. Of the citations a run files under a theme, how many genuinely support that theme, judged by a person reading the quoted span in its item. The mechanical verbatim check in the verify subcommand proves a span exists in the source. Audit answers the different question of whether the span means what the theme claims. Phase 1 showed why both checks exist. Every citation in both test iterations passed the verbatim check, while the human score moved from about 68 percent to about 84 percent between iterations. A tool that ran only the mechanical check would have certified the worse run.

## The sample is deterministic

An audit that samples differently each time cannot be argued with or repeated, so the sample is a stated algorithm, not a shuffle. For each theme, audit takes the theme's citations in the order the report emits them, which is sorted item id order, and selects evenly spaced entries across the full list to reach the sample size. The sample size per theme is ten, or the full citation count when the theme carries fewer. The same folder, taxonomy, and labelling state always produce the same sample, and the audit output names the corpus hash and taxonomy hash it sampled from. Anyone can rerun audit and receive the identical worksheet.

## The worksheet and the scoring

Audit writes two files. The file audit.md is the human worksheet, one section per theme, each sampled citation quoted in full with its item id and enough of the surrounding item text to judge it, and a blank verdict line per citation. The file audit.jsonl carries the same sample in machine readable form, one object per citation with the item id, theme id, span, and an empty verdict field.

The scorer writes one of three verdicts per citation. The verdict support means the span genuinely supports the theme. The verdict against means it does not, whatever the reason, whether wrong referent, wrong sense of a word, or a meta remark such as a review edit marker. The verdict unclear means the span is too truncated or ambiguous to judge, and unclear counts as against for the gate, because a citation a reader cannot judge is a citation the report should not lean on. Scoring is a human act in version 1. A model may pre score a worksheet when the user asks their own chat tool to, and the human still owns the verdicts that go into the score file, for the same reason the freeze confirmation is human.

## The gate computation

Precision per theme is supports divided by sampled, with unclear counted in the denominator and never in the numerator. The gate is four in five, a precision of 0.80, applied per theme and never as a global average, because Phase 1 showed a global average hiding a theme at two in five behind themes at five in five. Audit prints a score table with one row per theme, giving the theme id, the sampled count, the support count, the precision, and pass or fail against the gate. Themes with fewer than five citations in total are reported as unmeasured rather than passed, and an unmeasured theme cannot ship in the default taxonomy on the strength of the lexicon alone.

The score file is a record. A completed audit, meaning the worksheet with its verdicts and the score table, is kept in the repository or beside the taxonomy it measured, so a later reader can see which sample a shipped theme passed on and re-score it if they doubt the verdicts.

## Where audit runs

Audit runs in three places. At taxonomy release time, against the golden corpus, before a default taxonomy version ships. At propose freeze time, against the draft labelling file, where freeze refuses a failing theme. And on demand, against any folder a user wants measured, which is how the first dusk.fm run will be judged before its counts are quoted anywhere.

## Build notes

Audit lands in two parts. The sampling, worksheet, and score computation land with the lexicon counter in slice 1, because the taxonomy 0.2 precision scoring is part of slice 1 acceptance. The freeze integration lands with propose in slice 4. Audit needs no model, no network, and no clock, and it inherits the core determinism rules in full.
