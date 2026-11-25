# Slice 7. Exact near duplicate detection at export scale

The near duplicate check compared every pair of canonical
items. On a folder of hundreds that is nothing. On an
export of tens of thousands it is the whole run time, spent
on pairs that share almost nothing.

## What landed

- The pairwise pass in crates/earful-core/src/report.rs is
  now a candidate pass over a token index. A pair sharing
  no token has similarity zero, so only items sharing a
  token are ever compared. A size ratio check skips the
  rest, since similarity at or above 0.90 needs the smaller
  token set at least nine tenths the larger.
- Every surviving candidate still gets its exact Jaccard
  computed, so the flagged pairs are the same pairs, in the
  same order, that the all pairs pass produced. The
  prefilters are stated in the code with the reason each is
  sound.

## Tests

- A unit test builds a ten item corpus with a pair at the
  threshold and several near misses, computes the expected
  pairs by brute force inside the test, and asserts the
  finder returns exactly those pairs.
- The golden corpus numbers are unaffected. The rerun proof
  recorded zero near duplicate pairs on the Phase 1 corpus,
  and the count path's other outputs never touched this code.
