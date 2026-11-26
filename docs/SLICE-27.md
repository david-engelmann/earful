# Slice 27. The shipped taxonomy pinned to its record

Taxonomy 0.2 is a measured artefact, and nothing stopped
the shipped file and its audit record from drifting
apart silently.

## What landed

- A core test loads the actual taxonomy/v0.2.json from
  the repository tree on every build, parses it, and
  asserts the version and the eight theme ids in file
  order, the state the audit record documents.
- The documents index gains its slice 27 line.
