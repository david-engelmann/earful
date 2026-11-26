# The labelling path, worked end to end

The getting started example counts what a lexicon can see. This one shows what happens to the rest. Eight synthetic items sit in the corpus folder. Three report crashes in the words the taxonomy knows. Five report billing problems in words it does not, because none of them says invoice, billing, or charged. A plain count finds the three crashes and leaves the five untagged.

The labelling path covers them without teaching the counter any new words. Run the commands from the repository root.

First the worksheet. Propose lists the untagged items with their labelling fields blank.

```
cargo run -p earful-cli -- propose --folder examples/labelling/corpus --taxonomy examples/labelling/taxonomy.json --out /tmp/earful-labelling
```

A person or a model then labels the items outside the tool, by filling the theme, the cited span, and their name on each line. That work is already done in draft.jsonl in this folder, so the example can run. In real use the draft is the worksheet with those fields filled in.

Then the measurement. Audit-labels draws the sample to score, and a scorer records a verdict on each sampled line. Here every sampled span genuinely supports its theme, so every verdict is support.

```
cargo run -p earful-cli -- audit-labels --labels examples/labelling/draft.jsonl --folder examples/labelling/corpus --taxonomy examples/labelling/taxonomy.json --out /tmp/earful-labelling-audit
```

Open audit.jsonl in the out folder, set each verdict field to support, and save the result as verdicts.jsonl in the same folder. Then freeze, which runs the measurement, reports billing at precision 1.00 on its five sampled citations, and asks for the word freeze typed at the prompt before it writes anything.

```
cargo run -p earful-cli -- freeze --draft examples/labelling/draft.jsonl --folder examples/labelling/corpus --taxonomy examples/labelling/taxonomy.json --verdicts /tmp/earful-labelling-audit/verdicts.jsonl --reviewer "Example Reviewer" --out /tmp/earful-labelling/frozen.jsonl
```

Finally count with the frozen file. Billing now counts five items from the labels, crashes still counts three from the lexicon, nothing is untagged, and the report prints the frozen file hash so the run can be reproduced and checked.

```
cargo run -p earful-cli -- count --folder examples/labelling/corpus --taxonomy examples/labelling/taxonomy.json --labels /tmp/earful-labelling/frozen.jsonl --out /tmp/earful-labelling-count
```

A test in the command line crate runs this whole flow on every build, so the numbers above cannot drift from what the tool actually produces.
