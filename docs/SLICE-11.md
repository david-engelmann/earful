# Slice 11. A determinism stress test for the core promise

The product promise is byte for byte reproducibility, and
the existing proof is a two run comparison on one fixture.
Hash maps and hash sets carry random per instance state,
so any output path that iterated one would drift between
runs, and a two run comparison on a small fixture is a
thin probe for that leak.

## The audit behind it

Every output path in the core was read for hash iteration.
Theme accumulation is a BTreeMap. Corpus hashing walks
the item slice. Export source counts are first appearance
order in a vector. Audit and propose accumulation are
BTreeMaps. The hash collections that remain are lookup
only, and the one place keys are iterated, the near
duplicate candidate pass, sorts them first. No live drift
was found. What was missing was a test that would catch
the leak the day someone introduces it.

## What landed

- A core test builds a six theme taxonomy and a 120 item
  corpus from a fixed seed generator inside the test, with
  a shared vocabulary so the near duplicate path is
  exercised as well as the counters.
- It counts the corpus 25 times with fresh structures each
  time and asserts the canonical JSON and the Markdown are
  byte identical on every run.
