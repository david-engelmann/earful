# Earful design record, Phase 1 gate

Seat 6413cd79. Written 2026-11-20. Status is a gate report for Main Muse. No repository has been created and no code beyond the throwaway test prototype exists.

## What the product is

Earful is a local tool that reads a folder of feedback and produces theme counts where every theme carries verbatim cited examples from the source items. A rerun on the same folder with the same taxonomy reproduces the output byte for byte. There is no hosted service and no dashboard. The output is a machine readable file plus a human summary, and a reader can audit any number in either file down to the quoted line in the source item.

Inputs named in the brief are app reviews, support mail exports, GitHub issues, and survey free text. The first proving ground is David's dusk.fm feedback once an export exists. No dusk.fm export was present on this box during Phase 1, so the failure test ran on a public review corpus instead. That substitution is flagged as a scope note below.

## Name verification, exact checks run 2026-11-20

The working name at launch was groundswell. David rejected that name set the same day and this seat is now earful. Both names were checked exactly, and the groundswell results are kept here because they confirm the rejection was well founded.

Groundswell results. The GitHub name david-engelmann/groundswell returned 404 and was free. The crates.io name returned 404 and was free. The PyPI name returned 404 and was free. The npm name is taken by an active package at version 1.0.1, created December 2025 and modified June 2026, described as a hierarchical workflow orchestration engine for AI agents. A prominent company also trades as Groundswell in corporate philanthropy software, with a G2 listing, 102 reviews, and enterprise customers named in broadcast coverage. Neither incumbent works in feedback intelligence, so the name was usable for a Rust crate, but the npm block and the philanthropy brand made it a weak pick. The rejection stands on evidence, not taste alone.

Earful results. The GitHub name david-engelmann/earful returned 404 and is free. The crates.io name returned 404 with the registry stating the crate does not exist, and it is free. That is the primary registry for a Rust first build and it is clean. The PyPI name is taken at version 0.5.6 by a small unrelated tool that provides Discord Rich Presence for Apple Music on Linux, with 23 releases. The npm name is taken at version 0.5.6 by a build utility from 2014 whose description is earful from unnecessary building. A GitHub search for the exact name returns 291 repositories, and the most starred carries 27 stars and is a music project. No incumbent works in feedback intelligence, voice of customer, or developer tooling at any scale. This matches Main's search clearance.

Verdict reported to Main. Earful is clean on the two registries this product will actually use first, namely the GitHub namespace and crates.io. The PyPI and npm blocks matter only if a Python or JavaScript distribution is ever wanted under the exact name, and the fallback in that case is a suffixed distribution name such as earful-cli while the product name stays earful. No trademark search has been run and none is claimed. A trademark check is owed before any commercial launch under this name.

## The perfection pass

### The strongest counter case

The counter case is strong and the design record states it before the design. Buyers pay Thematic around 25000 dollars a year and pay Dovetail and Productboard per seat because those products discover themes the buyer did not already suspect, using language models over the raw text. A lexicon counter can only find what its taxonomy author already imagined. It answers the question of how much of what we already suspect is present. It does not answer the question of what users are actually saying. Theme discovery through embeddings and clustering, as in BERTopic, and theme labelling through a model judge both attack the real discovery problem, and both break byte for byte reruns. BERTopic clustering is stochastic across runs and environments, and a model judge is not reproducible at all once the provider changes a model behind an unchanged name.

Our own failure test makes the counter case concrete. After the precision fix described below, 291 of 598 items carried no theme at all. The lexicon missed plain praise such as working great, missed stability praise such as most stable, and missed a crash report phrased as stops working because the phrase list carried stopped working and does not work but not that inflection. A sceptic is entitled to say the tool counted the easy half and filed the rest as untagged.

A second counter case is scale. If dusk.fm feedback volume is small, a person can read all of it and counts add little. The product only earns its place when the folder is too large to read whole and too sensitive or too local to send to a hosted analyser, or when the same folder must be recounted over time and the counts must be comparable across runs.

### The sharpest form that survives

Earful is not theme discovery. Earful is theme accounting. The claim narrows to two sentences. Every number in the report traces to quoted lines a reader can check verbatim against the source folder. The same folder and the same taxonomy always produce the same report, byte for byte, on any machine, with no network and no model call.

Discovery still exists in the product, but it lives outside the counting core and its output becomes deterministic the moment it is frozen as data. A human or a model may propose labels once. Those labels are written to a labelling file that names the item, the theme, the verbatim span, and who or what produced the label and when. From that moment the counting core consumes the labelling file as data and never recomputes it. Reruns reproduce because the input includes the frozen labels. The boundary is stated on the face of every output, which names the taxonomy version and hash and the corpus hash, so a reader can tell exactly which inputs a count claims to describe.

This is the hybrid the brief anticipated. The counting half is deterministic and this record proves the rerun property for that half on a real corpus. The discovery half is honestly non deterministic, is optional, and is fenced off from the core by the freeze step.

### Scope amendments flagged to Main

One. The default taxonomy must pass a per theme precision gate before it ships. In the test, one theme built on the bare word update collected review edit markers and scored about two in five on its shown citations. Themes earn their place by measured precision on sampled citations, not by plausible phrase lists. The gate proposed is at least four in five sampled citations genuinely supporting the theme, which is the dossier bar.

Two. The first dusk.fm run needs an export that does not yet exist on this box. Creating that export is an integration task for the build phase and the Phase 1 test on a public corpus should not be read as evidence about dusk.fm feedback itself.

Three. A propose subcommand that drafts candidate labels for human freezing is proposed as part of version 1, gated on the same precision measurement. It is a scope addition relative to a pure lexicon tool and Main should rule on it explicitly. Without it the product is a counter for known themes only, and the record recommends saying exactly that on the README if the subcommand is cut.

## Design

### Input adapters

Version 1 adapters read a folder, never a live service. Each adapter produces the same normalised item stream and nothing downstream knows which adapter ran.

A text folder adapter reads .txt and .md files, one item per file, sorted by file name. A CSV adapter reads one named text column plus an optional id column, with the column names given in a small config file. A JSONL adapter reads one object per line with a text field and an optional id field. A mail adapter reads an mbox file or a folder of .eml files and takes the plain text body. A GitHub issues adapter reads a JSON export of issues and takes title plus body as one item. Every adapter assigns a stable item id. Where the source carries an id it is kept. Where it does not, the id is the position in sorted source order, so the same folder always yields the same ids.

Normalisation is part of the deterministic core and is pinned. Text is decoded as UTF-8 with replacement of invalid bytes recorded as a warning count. Line endings are normalised to LF. Nothing else is rewritten. The original bytes of every item are hashed with SHA-256 and the corpus hash in the output is a hash over the sorted list of item ids and item hashes. Matching works on a lowercased copy made with Unicode lowercase rules from the Rust standard library, and citations always come from the original text, never from the normalised copy.

### The theme model and the determinism boundary

A taxonomy is a data file, versioned, in JSON. Each theme carries a stable id, a plain language label, a kind drawn from recognition, operational issue, or other, and either a phrase list for the lexicon counter or no phrases at all when the theme is served only by frozen labels. Phrase matching is whole token matching. A phrase hits only at token boundaries, so ads cannot match inside reads or headset and import cannot match inside important. The test section shows why this rule is load bearing.

An item may carry several themes. Counting is per theme over distinct items. An item counts once per theme no matter how many phrases hit.

The deterministic core is ingest, normalisation, matching, counting, deduplication, rendering, and verification. Given the same folder, the same taxonomy file, and the same optional labelling file, the core produces byte identical themes.json and themes.md on any machine. The core makes no network calls, reads no clock, uses no randomness, embeds no absolute paths, and sorts every collection it emits. Shares are computed as count over analysed items and rounded to four decimal places by a stated rule.

Outside the boundary sit taxonomy authoring and the optional propose step. Both may use a model. Neither runs during a count. Their outputs enter the core only as files whose hashes are printed in the report header.

A verification subcommand re-derives every count from the source folder and the taxonomy, checks that every citation span occurs verbatim in its source item, and fails with a nonzero exit when any check fails. Verification is mechanical and uses no model.

### Output format

The machine readable output is themes.json in a canonical form with sorted keys, two space indentation, UTF-8 without escaping non ASCII text, and one trailing newline. Its fields are the format name and version, the taxonomy version, the taxonomy SHA-256, the corpus SHA-256, the count of items in the folder, the count of items analysed after exact duplicate removal, the exact duplicate list pairing each duplicate id with its canonical id, the near duplicate pair list, the count and ids of untagged items, and one entry per theme. Each theme entry carries the theme id, label, kind, count, share, the sorted item ids, and up to five citations. Each citation carries the item id and the verbatim span.

The human output is themes.md. It opens with the analysed, untagged, duplicate, and near duplicate counts in plain sentences. Then one section per theme in count order, headed by the label with its count and share, followed by the same citations as quoted lines with their item ids. The Hertz action shape is kept as a sort order only. Recognition themes print before operational issues of similar size in an optional actions view, and no routing or ticketing is performed.

Citations are sentence spans from the source item. The splitter works on sentence punctuation and merges very short fragments into the following span so abbreviations do not cut a citation in half as often. One residual flaw from the test is recorded below rather than hidden.

### How a theme earns its place

A theme appears in the report only when its count reaches a minimum support of three items or one half of one percent of analysed items, whichever is larger. Below that line the theme is listed in themes.json under a suppressed section with its count, so nothing is silently dropped and the suppression rule itself is auditable.

Exact duplicates are found on normalised text and are excluded from counting. The first item in sorted order is canonical and every later duplicate is listed against it. Near duplicates are found by token set Jaccard similarity at or above 0.90 over canonical items. Near duplicate pairs are flagged in the output and still counted, because two users writing near identical complaints are two complaints. The flag exists so a reader can see when one copied text is inflating a theme. In the test corpus, exact duplicates numbered two and near duplicate pairs numbered zero at that threshold.

A theme also earns its place through the precision gate described in the amendments. The default taxonomy ships only themes whose sampled citation precision measures at least four in five. The measurement method ships with the tool as an audit subcommand that prints a deterministic sample of citations per theme for human scoring.

## Cheapest failure test, run 2026-11-20

### Corpus and method

The corpus is the public user_quality review set by Grano and coauthors, 288065 real Google Play reviews of F-Droid applications, downloaded from the project's GitHub repository. The sampling rule was stated before the run and is deterministic. Rows were scanned in file order and the first 600 rows with review text of at least 20 characters became the folder, one verbatim .txt file per review. The prototype pipeline is a standard library only Python script written for this test. It is a test harness, not the product, and the stack section below treats it that way.

The prototype ran a ten theme starter taxonomy covering crashes and stability, performance and speed, battery and resource use, ease of use and interface, missing features, ads and monetisation, privacy and permissions, sync and data handling, updates and regressions, and praise and recognition. The dossier test is a human sampling of the citations. Fewer than roughly four in five citations genuinely supporting their theme kills the concept.

### Iteration 1, naive substring matching

The first run analysed 598 items after removing two exact duplicates and left 264 items untagged. All ten themes cleared minimum support. Two runs produced byte identical outputs and the independent verifier passed all 50 shown citations as verbatim. Hand scoring of the 50 shown citations gave about 34 in 50 genuinely supporting their theme, which is 68 percent and below the dossier bar.

The failures were concentrated and mechanical. The ads theme counted 28 items and almost none were about ads, because raw substring matching found ads inside reads, headset, and threads. The sync theme counted seven items and only one was about data handling, because import matched inside important and saves matched saves time rather than saving data. The updates theme collected review edit markers such as a leading UPDATE line. One citation was truncated mid word because the sentence splitter cut at the abbreviation e.g.

### Iteration 2, whole token matching

The single change was whole token phrase matching, plus merging very short sentence fragments. The rerun was again byte identical across two runs, with themes.json at SHA-256 d081ae0479df4e444a2a9d9498a327695213b5b0de566d3d39c9b5ac83a35599, and the verifier again passed every citation as verbatim.

Counts moved honestly. Ads fell from 28 items to three, and all three shown citations were genuinely about ads. Sync fell below minimum support and was suppressed, because its first iteration count had been mostly false positives. Total themes fell from ten to nine. Untagged items rose from 264 to 291 of 598, which is 48.7 percent. Precision was bought with coverage, and the record reports both numbers rather than only the good one.

Hand scoring of the 43 shown citations in iteration 2 gave about 36 in 43 genuinely supporting their theme, which is about 84 percent and above the dossier bar. The remaining failures are instructive. The updates theme still scored about two in five, on review edit markers and a material design update that is an interface remark rather than a regression. One battery citation concerned a transmitter battery rather than the app. One performance citation praised a responsive developer rather than a fast app. One privacy citation was about tracking down bugs. One missing features citation is still truncated at the abbreviation, because the fragment merge only repairs short leading fragments and the truncated span itself was long. That splitter flaw is a known defect carried into the build backlog with its example attached.

### What breaks, stated plainly

Lexicon recall is the binding limit. Nearly half the folder carried no theme, and the untagged sample shows real signal being missed through inflection and paraphrase. Polysemy survives token boundaries, as the tracking and responsive cases show. Theme quality varies sharply by theme, so a single global precision number would hide the weak themes, which is why the gate is per theme. The concept survives its cheapest failure in narrowed form. The counting and citation machinery is sound and provably reproducible. A fixed lexicon alone is a weak discovery instrument, and the product must say so and must ship the frozen labelling path if it wants to cover what the lexicon misses.

## Stack proposal

The product is a single Rust binary named earful, with a workspace of two crates. The core crate earful-core holds ingest, normalisation, matching, counting, deduplication, rendering, and verification, and it uses only the standard library, following the fleet pattern that core crates stay std only. The binary crate earful-cli holds argument parsing, the adapters that need parsers for CSV, JSON, and mail formats, and file walking, and it may use external crates pinned in Cargo.lock. Rust is the fleet default and it fits this product for a specific reason beyond habit. Byte for byte reproducibility across machines is the product claim, and a compiled single binary removes interpreter version drift, hash randomisation, and locale dependent behaviour from the counting path in one move.

The Python prototype is the one exception and it is justified in writing here. It existed to run the cheapest failure test in an afternoon before any repository was created, it used only the standard library, and it is discarded at build time. No Python ships in the product. No JavaScript ships either, which is why the taken npm name does not block the plan. A Python distribution is not planned, which is why the taken PyPI name does not block the plan, and if one is ever wanted it will use a suffixed name.

Testing follows the house pattern. Unit tests cover matching boundaries, deduplication, canonical rendering, and the verifier. A golden test reruns the Phase 1 corpus through the Rust core and compares the full output against a frozen expected file, so the byte for byte claim is under continuous test rather than asserted. The three review checkpoint rule applies at the design gate, at the first working counter, and before any release.

## Access and next steps

No repository has been created. On Main's confirmation, the repo david-engelmann/earful is created and receives its own per repo SSH deploy key at creation through the stored credential, following the fleet pattern recorded in the deploy keys inventory. The repo is not added to the existing GitHub App installation, which is at its repo limit. The build order after confirmation is adapters and items normalisation first, then the lexicon counter with the verifier, then the golden corpus test, then the frozen labelling path, with the propose subcommand last and only if Main rules it in.

This record and the prototype outputs live in the seat workspace under refrain-phase1. Under the house rule that the workspace is temporary storage, this record needs its off box copy through Main's banking route in the same cycle it is confirmed.
