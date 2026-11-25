# Slice 6. Abbreviation aware sentence splitting

The oldest defect on the books, carried from the Phase 1
record. A citation could end mid abbreviation, because the
sentence splitter treated every full stop as a boundary and
the fragment merge could not repair a span that ended at
e. The Phase 1 corpus item-0001 citation for missing
features ended exactly there.

## What landed

- The sentence splitter in crates/earful-core/src/text.rs
  no longer treats a full stop as a boundary when the token
  before it is a single letter, an initial, or one of a
  small named set of common abbreviations, e.g, i.e, etc,
  vs, mr, mrs, dr, st, approx, fig, no, a.m, p.m, u.s, u.k.
- The trade is stated in the code where it is made. A
  sentence that ends in an abbreviation merges with the
  next one, so a citation may run long. It never truncates.
- The module doc comment that carried the known flaw now
  describes the rule that closes it.

## Tests

- A unit test holds the e.g. sentence in one span.
- A regression test carries item-0001 verbatim from the
  Phase 1 corpus and pins the citation to the full final
  sentence.

## Effect on the golden corpus

Counts are untouched, because matching is token based and
never depended on spans. Citations change where an
abbreviation sat inside a cited sentence, so a fresh count
over the Phase 1 corpus will not byte match the rerun proof
artefact, which stands as the record for the earlier head.
