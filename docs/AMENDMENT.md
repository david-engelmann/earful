# Amendment addendum, Main's instruction of 2026-11-20 16.49 EST

This addendum answers the two points the Phase 1 gate report covered only in general form. The determinism boundary and the citation hand check were already in the gate report and the design record in full. The boundary in one sentence is that the counting and citation core is byte for byte reproducible and never calls the labelling half, while any model assisted labelling is performed once and enters later runs only as a frozen labelling file whose hash is printed in the report. The hand check scored both test iterations citation by citation and is reported with its counts in the gate report.

## First user, first dataset, exact v1 input shapes

The first user is David, counting dusk.fm feedback, with his connected app work as the design partner. The tool is designed against that folder first and every other folder second.

Version 1 supports exactly three input shapes and refuses the rest.

Shape one is a folder of plain text files. One feedback item per .txt or .md file, UTF-8, any file name, read in sorted file name order. This is the shape a manual export, a mail save, or a review scrape naturally lands in, and it is the shape the Phase 1 test proved.

Shape two is a single CSV export with exactly these columns, in any order, named in a header row. A column named text carrying the feedback body is required. Columns named id, source, date, and rating are optional and are carried through to the item record when present and ignored when absent. Any other columns are ignored. This is the shape app store and survey exports arrive in.

Shape three is a single JSONL file, one object per line, with a required string field named text and optional fields named id, source, date, and rating. This is the shape a dusk.fm database export or a GitHub issues dump can be produced in with one query or one API call, and it is the recommended shape for the first dusk.fm run.

Refused in version 1, by name, are mbox and .eml mail archives, PDF and Office documents, direct API connectors to any store or helpdesk, and live database connections. The design record listed mail and GitHub issues adapters without a version fence, and this addendum corrects that. A GitHub issues dump is supported only when it has already been exported to shape three by the user. Mail is supported only when messages have already been saved as shape one text files. Each refused shape can be converted to a supported shape with tools the user already has, and the refusal keeps the v1 ingest surface small enough to verify byte for byte behaviour on all of it.

The first dusk.fm run still needs the export itself, which does not exist on this box. The build phase task is unchanged, and shape three is the shape that export should be produced in.

## The scripts plus a prompt counter case, answered directly

The strongest form of this counter case is that a folder of feedback plus a chat model and one good prompt already produces themes, counts of a sort, and example quotes, in an afternoon, for free, with no new tool at all. That is true for a folder small enough to paste and for a report nobody needs to reproduce. The dossier itself named this baseline, since its failure test asked whether the cited report beats the uncited summary the same model produces from the same feedback pasted raw.

What earful adds survives that argument on four grounds, and only on those four.

First, reproduction. A pasted prompt run cannot be rerun. The model changes behind an unchanged name, sampling varies, and the context window reorders what the model attends to. Two runs a month apart cannot be compared, so a theme count moving from 30 to 45 cannot be shown to be the feedback changing rather than the reader changing. Earful counts are comparable across time because the counting never changes between runs unless the taxonomy file changes, and the taxonomy change is itself a versioned, hashed, visible event.

Second, scale. A paste ends at the context window. The test corpus held 598 items and real folders hold tens of thousands. Earful reads the whole folder every run at no per item cost.

Third, audit. A model summary quotes from memory of the paste and its quotes drift, shorten, and occasionally invent. Every earful citation is checked mechanically against the source bytes by the verify subcommand, and the Phase 1 test showed that check passing on all citations in both iterations while the human hand check separately scored whether the verbatim quotes genuinely supported their themes. Those are two different checks and the product keeps both.

Fourth, custody. The folder never leaves the machine. Support mail and unreleased product feedback are exactly the corpora a team may not paste into a hosted chat, and the commercial floor for a hosted analyser in this space is around 25000 dollars a year.

Where the counter case wins, the record concedes it. For a one off read of a small folder that nobody will recount or audit, the prompt is enough and earful adds ceremony. The product is for the folder that comes back every month and whose numbers somebody has to defend.
