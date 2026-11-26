# Slice 13. Release readiness, metadata and honest usage text

Launch prep, two small surfaces that a first user or a
package index meets before any code.

## What landed

- Both crates now carry the publish metadata a package
  index needs, repository and homepage inherited from the
  workspace package table, and per crate keywords and
  categories. Nothing about publishing itself changes.
  Putting the crates on an index stays a separate act with
  its own credentials.
- The usage text no longer claims every command takes a
  folder and a taxonomy. Score takes a sample and a
  verdicts file, and check-export takes an input alone.
  It now says most commands, which is true, and the stray
  duplicated usage line above it is gone.
