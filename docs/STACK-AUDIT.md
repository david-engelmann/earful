# The stack audit, 2026-11-25

Before the stacked branches are handed over for landing, the stack was audited mechanically against the hosted state, in three checks. This record states what was checked, what held, and the two findings the landing needs to know.

## The tree at the top

The tree at the top of the stack, the head of slice/taxonomy-pinned, was listed in full. It holds 92 files. Every expected file is present. The design records, the amendment, the audit contract, the propose design, the export contract, the taxonomy guide, the formats reference, the documents index, the security note, the changelog, all twenty seven slice documents, the shipped taxonomy, and all three worked examples with their full corpora. Nothing the slices claimed to add is missing.

## The ancestry

The first parent chain was walked from the top back to the repository's first commits, thirty nine commits. Every branch tip in the stack is an ancestor of the top except two, and both exceptions are explained below. Every other slice stands in one line, in the order it was built.

## The verdicts

Every branch head in the stack carries a hosted run on its exact head, and every one of those runs completed with success, including the repair heads that superseded failed first attempts on slices 7, 9, 14, 22, 23, and 27. The failures and their repairs are named in the commit messages on those branches. The one exception is the proof branch below, which predates the branch gating pattern and was gated locally in its own window, as its document records.

## Finding one. The proof branch is already contained

The proof branch, proof/rerun-proof-0.2, is not an ancestor of the top, because the importer slice was staged on remote main rather than on the proof commit. Its whole contribution is the rerun proof document and the audit record correction, and both files stand in the top's tree with blob hashes identical to the proof branch's own. Landing it as its own pull request would duplicate content the stack already carries. It should be closed unmerged when the stack lands, or skipped in the order.

## Finding two. The labelling cousin joint

The final head of slice/labelling-example is not an ancestor of the top either, for the reason reported when it happened. Its second repair was staged after the export example had been built on its first repair. The five files that differ between that head and the top are exactly the files the later slices went on to change, and the top carries each of them in a later form that includes the labelling head's content. Nothing exists at that head that the top lacks. Landing by tree is clean. Landing the labelling branch after the export branch would present an empty or conflicting diff, and it should land before it, or its final repair commit should be skipped as contained.

## What the audit did not do

It did not re-run the suites, re-read the code line by line, or re-score the taxonomy. The hosted gates and the slice records carry those. This audit checked the stack as a stack, that the files are where the records say, that the line is one line where it claims to be, and that every head is green on its own run.
