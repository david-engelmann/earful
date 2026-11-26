# Getting started

A tiny worked example. Eleven short feedback items sit in the corpus folder and a four theme taxonomy sits beside them. The items are synthetic and written for this example.

Run the count from the repository root.

```
cargo run -p earful-cli -- count --folder examples/getting-started/corpus --taxonomy examples/getting-started/taxonomy.json --out /tmp/earful-example
```

The run writes themes.json and themes.md into the out folder. Three themes reach the minimum support of three items, crashes and freezes with three, praise with three, and wishlist with three. Battery matches one item and is listed under below minimum support rather than dropped. One item matches no theme and is reported as untagged.

Then verify the result against the same folder and taxonomy.

```
cargo run -p earful-cli -- verify --folder examples/getting-started/corpus --taxonomy examples/getting-started/taxonomy.json --themes /tmp/earful-example/themes.json
```

Verify recomputes everything and prints ALL CHECKS PASS when the files match the recomputation. A test in the command line crate runs this example on every build, so the numbers above cannot drift from what the tool actually produces.
