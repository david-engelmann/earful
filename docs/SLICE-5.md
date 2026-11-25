# Slice 5. The receiving side importer

The receiving half of the dusk.fm integration. Slice 4
established the export contract and its checker. This slice
builds the importer that takes an export file in at one end
and produces a verified themes report at the other, refusing
anything that does not pass the contract.

## What landed

- An import subcommand taking an export file, a taxonomy,
  and an output folder, with the same optional frozen
  labelling file as count.
- For the JSONL shape, the slice 4 contract check runs
  first. Any contract failure, or any decode warning, refuses
  the import before a single item is counted, with every
  break named. The CSV fallback shape carries no contract
  check in version 1 and imports through the adapter.
- The import then counts, writes themes.json and themes.md,
  and runs the full verify over its own output before
  reporting success.
- The reconciliation from the export specification runs
  last, as a core function beside the contract checker. The
  export line count must equal the items read, the analysed
  count must equal the export count minus the exact
  duplicates listed, and the decode warnings must be zero.
  The import prints all four numbers and passes or fails on
  them. The fourth reconciliation point in the spec, the
  scored audit worksheet, stays a human step served by the
  audit subcommand.

## Tests

- A core unit test for the reconciliation, accepting a
  matching report and naming both failures on a mismatched
  line count.
- One CLI integration test imports the specification's two
  synthetic lines, checks the reconciliation passes and the
  totals land, and confirms an export with an unlisted
  source is refused with no report written.

## Landing note

Direct pushes to main are now refused by a repository
ruleset that arrived with the fleet Trunk enrollment, so
this slice lands by branch and pull request, the same route
as the rerun proof branch.
