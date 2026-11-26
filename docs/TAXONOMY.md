# Writing a taxonomy

A taxonomy is the list of themes earful counts. It is a JSON data file, versioned like code, and it decides everything the report can say. This guide covers how to write one that counts honestly. The small file in examples/getting-started is a complete working model, and taxonomy/v0.2.json in this repository is a larger one that was measured before it shipped.

## The shape

The file carries a version string and a list of themes. Each theme carries an id in snake case, a label written for the report's readers, a kind naming what sort of theme it is, and the phrases that file an item under it. The kinds in use are operational_issue for things going wrong, feature_request for things asked for, praise and recognition for things going right. Kind is a label for grouping, nothing more. The counter does not branch on it.

## How matching works

Matching is whole token and multi label. An item counts toward a theme when one of the theme's phrases appears in it as whole words, in order, ignoring case and punctuation. One item can count toward several themes, and it counts once per theme however many of its phrases hit. A phrase that is part of a longer word never hits, so the letters ads inside reads or headset file nothing. That rule is the reason the tool exists in this shape. The first prototype matched substrings and counted twenty eight items under advertising, nearly all of them false, on words like reads and threads.

## Choosing phrases

Start from real items. Read a sample of the feedback and lift the words people actually use, in the forms they use them. Prefer several narrow phrases over one broad one. A broad phrase files items the theme does not mean, and the audit will catch it only after it has inflated a count someone may already have quoted. When two themes compete for the same phrase, give it to the one a reader would expect and let the other find its own words. Expect an untagged share. The measured run of taxonomy 0.2 left just under half its items untagged, and that figure is reported, not hidden. A taxonomy that files everything is usually filing things wrongly.

## Support and suppression

A theme counts only when it reaches the minimum support, the larger of three items and half a percent of the items analysed. A theme under that line is listed in the report as suppressed, with its count, never silently dropped. Small pools are where noise lives, and a count of one proves nothing about a product. If a theme matters at small numbers, it matters as reading, not as accounting, and the citations are there to read.

## The gate for a default taxonomy

A taxonomy that ships as a default carries a stronger obligation than one written for a single run. Before a theme ships in a default taxonomy it is measured by the audit. A scorer reads a sample of its citations and records whether each one genuinely supports the theme, and the theme must clear four in five. A theme whose total pool is under five citations cannot be measured at all and does not ship in a default on the strength of its phrases alone. The full contract is in docs/AUDIT.md, and the worked measurement for the current default is in docs/AUDIT-RECORD-0.2.md.

## Versioning

Change the version string whenever the themes or phrases change, and treat the change like a code change, reviewed and recorded. Reports print the taxonomy hash alongside the corpus hash, so any two runs can be compared honestly, and a count quoted from an older taxonomy can be told apart from a count under the current one. When a theme is removed, say why in the commit. Taxonomy 0.2 removed two themes whose pools sat under the measurement floor, and the record says so.
