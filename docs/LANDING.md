# Landing the stack

The build stands on stacked branches over main at e4b18fe0. This guide is the landing order, with each branch's head and the hosted run that gated it, read back from the repository on 2026-11-25. Land them in this order, each pull request based on the branch before it or on main for the first, through the merge queue. Every head below completed its hosted run with success on the exact head named.

## Skip one

- proof/rerun-proof-0.2 at 2cfd335798. Do not land it. Its whole contribution already stands in the stack's trees with identical blob hashes, as docs/STACK-AUDIT.md records. Close it unmerged.

## The order

1. slice/importer at 767cd1c3e5, run 45528151935. The receiving side importer and the four number reconciliation.
2. slice/sentence-splitting at c887f403ec, run 45540932117. Abbreviation aware sentence splitting.
3. slice/near-duplicate-index at 2f0c6b7dc0, run 45542814693. Exact near duplicate detection at export scale, with its repair.
4. slice/json-depth-limit at b8549b153f, run 45542885024. The JSON parser nesting limit.
5. slice/bom-stripping at 7adcbd124d, run 45547408770. Byte order marks stripped, with its format repair.
6. slice/verify-markdown at b3446c92bf, run 45550860341. Verify checks the Markdown rendering.
7. slice/determinism-stress at eb3a39caa7, run 45553949766. The determinism stress test.
8. slice/readme-current at 2dbc8ebbc5, run 45555603145. The README brought current.
9. slice/release-readiness at a77db3816d, run 45557399758. Crate metadata and the corrected usage text.
10. slice/getting-started at 1757970dd5, run 45566999834. The getting started example, with its assertion repair.
11. slice/decisions-current at c689721f63, run 45567049565. The decisions record brought current.
12. slice/design-postscript at 3efb9f9248, run 45577036124. The postscript on the design record.
13. slice/changelog at fb19c7ec00, run 45577069182. The changelog.
14. slice/folder-ingest at 43cef24aa0, run 45578978587. Folder ingest hardening.
15. slice/docs-index at 9c3d613e92, run 45579130983. The documents index.
16. slice/readme-pointers at 1e5927a30c, run 45586220795. The README points at the example and the index.
17. slice/edge-inputs at 0041eb1d2e, run 45586425690. Edge input tests for the counter.
18. slice/labelling-example at d0c4ce7f01, run 45596577706. The labelling path worked example, with its two repairs. Land this one before the next.
19. slice/import-example at ccfff220e2, run 45597087052. The export path worked example, with its two repairs.
20. slice/taxonomy-guide at bc8ab7bc3e, run 45607278094. The guide to writing a taxonomy.
21. slice/formats-reference at 2d76283d7c, run 45607620861. The file formats reference.
22. slice/security-note at 046c684f6d, run 45616869053. The security note.
23. slice/taxonomy-pinned at 7486d67d79, run 45617025665. The shipped taxonomy pinned to its record, with its format repair.
24. slice/stack-audit at 233b381f16, run 45622454441. The mechanical audit of the stack.
25. slice/landing-guide, this guide's own branch, lands last. Its head is the stack top when read, and it carries no separate gate beyond the same hosted run on its head.

## After the landing

When the last branch lands, main's tree should match the landing guide branch's tree exactly, file for file. Check it by listing both trees and comparing, the same mechanical check the stack audit ran. The branches can then be deleted from the landed end, never a branch a later pull request still bases on. What remains open after the landing is the one programme item no branch carries, the first run against a real dusk.fm export, specified in docs/DUSKFM-EXPORT.md and rehearsed in examples/import.

## Completion, recorded 2026-11-26

The landing ran as this guide prescribes, with one stop. The first four slices landed in order at the heads named above. The bom-stripping pull request then stood conflicting against main with a fourteen file diff, because the earlier landings had reached main as new commits and its merge base had not moved. It was closed unmerged, and from there each remaining slice's tree was carried unchanged onto a fresh branch off the then current main, so each pull request diffed as exactly its own slice. Every landing was verified tree by tree against its slice as it went. The final pull request, number 27, merged on a green gate, and main's tree read 96fe8fc4678d2c4db0194b9208c8faf6b2ccc8fe, identical to this guide's branch tree, file for file, under the closing check above. The carry branches were deleted from the landed end as they went, and the slice branches were deleted by the seat that built them once the landing closed. Only main remains.
