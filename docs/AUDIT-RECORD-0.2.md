# Audit record for taxonomy 0.2

This is the kept record the audit design requires, the scored sample a shipped taxonomy passed on. It covers taxonomy version 0.2.0 at taxonomy/v0.2.json, measured on the Phase 1 golden corpus. The corpus is the deterministic sample of public Google Play reviews built in Phase 1, 600 files in folder order, 598 canonical items after two exact duplicates are excluded. Method evidence only. Nothing here is evidence about any real product's feedback.

## How this measurement was taken

The sample follows the audit design exactly. Per theme, the pool is every citation the counter files, in item order, and the sample is ten evenly spaced entries across the pool, or the whole pool when smaller. Every sampled citation was scored by hand from the span read in its full item, with verdicts of support, against, or unclear, and unclear counting against the gate. The gate is four in five per theme. Themes under five citations in total are unmeasured.

One honesty note on tooling. The sample for this record was produced with the Phase 1 prototype logic, which implements the same matching, sentence, and sampling rules, because the Rust build's output writes are currently refused by this machine's intermittent write fault. The Rust audit subcommand implements the identical algorithm, and reproducing this sample and score table with the Rust binary is an owed cross check, recorded in the slice 1 report. The verdicts are human judgements and do not depend on which implementation printed the worksheet.

## Round one scores

Crashes and stability, pool 122, sampled 10, supports 10, precision 1.00, pass. Praise and recognition, pool 180, sampled 10, supports 10, precision 1.00, pass. Interface and design, pool 15, sampled 10, supports 10, precision 1.00, pass. Missing features and requests, pool 40, sampled 10, supports 8, precision 0.80, pass at the gate, with two failing phrases identified. Battery and resource use, pool 28, sampled 10, supports 2, precision 0.20, fail. Performance and speed, pool 9, sampled 9, supports 3, precision 0.33, fail. Privacy and permissions, pool 6, sampled 6, supports 4, precision 0.67, fail.

Why the failures failed is the useful part. In this corpus the bare word battery mostly names the domain of the apps under review, which are battery widgets and monitors, so the theme measured the app's subject rather than any battery concern. Speed words did the same for charging hardware, with slow and fast describing chargers, cords, and ports. Bare permission caught folder permissions and a settings request alongside genuine privacy concerns.

## Redesigns and round two scores

Battery was redesigned to consumption phrasing, then narrowed once more when bare drain proved to name the measurement the widgets display rather than consumption. Final pool 5, sampled 5, supports 5, precision 1.00, pass. Performance was redesigned to application performance phrasing, and its pool fell to one item, under minimum support, so the theme is suppressed in output on this corpus and asserts nothing. Privacy was redesigned to concern phrasing and its pool fell to four, all four genuine. Ads stood at a pool of four, three genuine and one unclear in a garbled review. Missing features had its two failing phrases removed, the wish it phrase that fired on a wish that the app worked, and the no way to phrase that fired on a usability failure, and its new sample scored ten in ten, pool 34, precision 1.00, pass.

## The pull decision

The audit contract says an unmeasured theme cannot ship in the default taxonomy on the strength of the lexicon alone, and the gate binds counted themes, meaning themes that appear in a report with a count a reader will quote. Privacy and ads were counted themes, each above minimum support at four items, with pools under the five citation measurement floor. Both are pulled from taxonomy 0.2.0 and recorded here with their scored samples, privacy four in four genuine and ads three in four. Either returns to the default taxonomy when any corpus gives it five citations to measure. Themes under minimum support, performance at one, sync at one, and regressions after update at two, make no counted claim and stay in the taxonomy as suppressed entries.

## Final state of taxonomy 0.2.0

Five counted themes, each measured at precision 1.00 on this corpus. Crashes and stability at 122 items, praise and recognition at 180, missing features and requests at 34, interface and design at 15, and battery and resource use at 5. Three suppressed themes listed with their counts. Items with no theme stand at 284 of 598, which is 47.5 percent, up from 42.8 percent under the wider first draft of this taxonomy. That rise is the measured price of the precision gate on this corpus, and it is the gap the propose and freeze path exists to close under human control.

Correction, 2026-11-24. The untagged figure above is stale. The rerun proof in docs/RERUN-PROOF-0.2.md recomputed the final taxonomy with both the Rust build and the Phase 1 prototype on identical corpora, and both produce 283 untagged items, 47.3 percent, with identical untagged sets. The 284 was written during the redesign rounds, before the final battery narrowing settled the shipped taxonomy bytes. Every other number in this record reproduces exactly.

## Borderline verdicts, recorded

Three calls are worth a second reader's eye. One missing features citation was scored support on the full item, a wish for Tasker integration, where the quoted span alone reads vaguely. One missing features citation is the known truncated span ending at the abbreviation in item-0001, scored support because the request is still judgeable in context, with the truncation defect already on the backlog. One battery citation about an implied 250 hour battery life was scored support because battery life is its subject, where a stricter reader could call it a complaint about a reading. No verdict change among these three moves any theme across the gate except the battery one, and battery passes at five in five only with it, so a stricter rescoring would pull battery to four in five on a sample of five, still a pass at 0.80. The record stands either way.
