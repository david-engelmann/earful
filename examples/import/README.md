# The export path, worked end to end

The other examples start from a folder of text files. A real run starts from an export, one file in the shape the export contract in docs/DUSKFM-EXPORT.md specifies. This example is that shape at rehearsal scale. The export holds nine synthetic lines with ids in order, sources from the contract list, and dates and ratings where the source system had them. One line repeats another word for word, as real exports do.

Run the commands from the repository root. First the contract check, which validates the ids, sources, dates, and ordering before anything is counted.

```
cargo run -p earful-cli -- check-export --input examples/import/export.jsonl
```

Then the import, which runs the same check, counts the export, verifies its own output, and reconciles the numbers before it reports success.

```
cargo run -p earful-cli -- import --input examples/import/export.jsonl --taxonomy examples/import/taxonomy.json --out /tmp/earful-import
```

The reconciliation prints four numbers and they agree. Nine export lines, nine items read, eight items analysed, one exact duplicate set aside, and no decode warnings. The count itself finds crashes at three items. Praise, missing features, battery, and sync each match one item, under the minimum support of three, so the report lists them as suppressed rather than counted, and one item matches nothing and is reported untagged. That honesty is the point of the exercise. A first real export run is accepted on exactly these terms, the contract holding and the four numbers reconciling before any count is quoted.

A test in the command line crate runs this example on every build, so the numbers above cannot drift from what the tool actually produces.
