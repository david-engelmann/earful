# The dusk.fm export integration, specification

Status is a build ready specification, written 2026-11-21 under Main's press order. The first user is David and the first dataset is dusk.fm feedback, with the connected app work as the design partner. This specification defines exactly what the export must look like so the first real run needs no design decisions on the day. The export itself does not exist yet. Producing it is a build phase task on the dusk.fm side, and nothing here requires access to any dusk.fm system from this repository.

## The primary shape is JSONL

The export is one file named duskfm-feedback.jsonl, UTF-8, LF line endings, one JSON object per line, lines sorted by id ascending. Sorting is part of the contract, because sorted order is what makes item ids and citation order stable across exports of an unchanged dataset.

Each object carries these fields. The field text is required, a string holding the feedback body exactly as the user wrote it, with no trimming, no case changes, and no markup stripping on the export side, because earful normalisation is pinned and any earlier rewriting would make citations disagree with the source system. The field id is required for this integration, a string holding the feedback record's own identifier in dusk.fm, so a citation can be traced back to the record it came from. The field source is required, a string drawn from a short list the export owns and documents, with starting values of support, review, in_app, and email. The field date is optional, a string in ISO 8601 UTC form, used for later time slicing and never for counting. The field rating is optional, a number, carried through to the item record when present.

Two example lines follow. They are synthetic, written for this specification, and are not real user feedback.

{"id": "fb-0001", "source": "in_app", "date": "2026-11-01T14:23:00Z", "text": "The player stops playing when the screen locks and I have to reopen the app."}
{"id": "fb-0002", "source": "support", "date": "2026-11-02T09:05:00Z", "rating": 2, "text": "Sync between my phone and desktop has not worked since the last update."}

## The fallback shapes

When a JSONL export is not practical for a given source, the CSV shape is the fallback. The file carries a header row with a required column named text and optional columns named id, source, date, and rating, matching the JSONL fields one for one. Standard CSV quoting applies, and embedded newlines inside a quoted text field are one item, not several.

Ad hoc material, such as a handful of support messages saved by hand, uses the folder shape. One .txt file per message, named with the record id when one exists, read in sorted file name order. The folder shape is for tens of items, not thousands, and the first full dusk.fm run is expected on the JSONL shape.

Shapes outside the version 1 fence stay refused for this integration too. No direct database connection, no API connector, and no mail archive reading. The export is a file, produced on the dusk.fm side by whatever query or admin export already exists there, and earful begins at the file.

## Custody and privacy terms of the integration

The export file never leaves David's machine, and earful makes no network calls, so the integration adds no new custody surface. Earful offers no PII redaction guarantee in version 1, and this specification does not pretend otherwise. The control is custody, not redaction. The export is stored with the same care as the source data, the themes output quotes short spans from it, and anyone publishing a themes file is responsible for the spans it carries, exactly as they would be publishing the feedback itself.

## Reconciliation checks for the first run

The first run is accepted only when four numbers reconcile in the run report. The export line count equals the items in folder count in themes.json. The analysed count equals the export count minus the exact duplicates listed, and every duplicate listed names a pair a person can open and agree is the same text. The warnings count from decoding is zero, or every warning is explained. And the audit worksheet for the run is scored before any count from the run is quoted, with any theme under the gate pulled or redesigned rather than explained away.

## Build readiness

On the earful side this integration needs nothing beyond the slice 2 JSONL and CSV adapters reading the fields above, which are already scoped. On the dusk.fm side it needs one export producing the file above from the feedback store, plus a decision on the source value list. Both are small, and neither waits on the other, so the integration can land the same week the export exists.
