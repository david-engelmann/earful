# The file formats

Earful speaks in files. This document specifies the ones it writes and reads, so another tool can produce or consume them without reading the source. The design reasoning lives in docs/DESIGN.md and docs/PROPOSE.md. This is the reference.

## The report, themes.json

The format name and version stand in the format field, currently earful-themes/0.2. The file is written canonically, keys sorted and layout fixed, so two runs of the same inputs compare byte for byte and a hash of the file means something.

The header fields name what was counted. Taxonomy version and the SHA-256 hash of the taxonomy file, the SHA-256 hash of the corpus, and the SHA-256 hash of the frozen labelling file, or null when no labelling file was used. Then the shape of the corpus. Items in folder is the number of items read before duplicate handling. Items total is the number analysed after exact duplicates were set aside. Items untagged counts the analysed items no theme claimed. Decode warnings counts the files whose bytes needed replacement characters on the way in. Exact duplicates lists each set aside item with the id of the canonical item it duplicates. Near duplicate pairs lists the pairs flagged at the similarity threshold, each with its two item ids.

Each counted theme carries its id, label, and kind, its count and its share of the analysed items, the ids of its items in corpus order, and its citations, each a verbatim span with the id of the item it came from. Up to five citations stand in the file per theme. Suppressed themes are listed with their counts, never dropped silently. The untagged item ids close the file.

The Markdown rendering, themes.md, carries the same facts in reading order and is verified alongside the JSON. A verifier that finds the two disagreeing fails the run.

## The labelling file

One JSON object per line. A label line carries the item id, the theme id, the cited span exactly as it appears in the item, the labeler's name, and a status. In a draft every line carries the status draft, and the counter refuses a draft outright.

A frozen file opens with a header line whose kind is earful-labelling-header. The header carries the status frozen, the SHA-256 hashes of the corpus and the taxonomy the labels were made against, the reviewer's name, and the precision measured at freeze time for each labelled theme, as a number or null when the theme was unmeasured. The counter refuses a frozen file whose hashes do not match the run in front of it. The freeze path that produces this file is specified in docs/PROPOSE.md.

## The worksheet

Propose writes the worksheet a labeller fills in. Its first line is a header whose kind is earful-worksheet, naming the taxonomy version and its themes with their labels and kinds. Each following line carries one untagged item in full, its id and its text, with the theme, span, and labeler fields blank and the status draft. A completed worksheet, with the header removed or kept, is a draft labelling file.

## The audit sample and verdicts

The audit commands write a sample file, one JSON object per line, each carrying a theme id, an item id, the cited span, and a verdict field left blank. A scorer fills each verdict with one of support, against, or unclear, reading the span against its item. The score and freeze commands read that file back. Unclear counts against the theme, by the contract in docs/AUDIT.md. The sample is drawn evenly across each theme's citations in report order, so the same report always draws the same sample.
