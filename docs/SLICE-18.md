# Slice 18. Folder ingest hardening

The folder shape had three silent behaviours the file
shapes did not share.

## What landed

- The extension match is case insensitive. A folder
  assembled on a case insensitive filesystem holds ITEM.TXT
  as readily as item.txt, and the old match dropped such
  files without a word.
- Directories are skipped, whatever their name. A folder
  named archive.txt inside the corpus no longer fails the
  whole run when the reader tries to read it as a file.
- Duplicate item ids are refused. Two files can share a
  stem, a.txt and a.md, and so share an id, and an id is
  how citations are attributed. The CSV and JSONL shapes
  already refused duplicate ids. The folder shape now
  refuses them too, with the same error.

## Tests

- A core test builds a folder in a temp directory holding
  an uppercase extension, a mixed case Markdown file, a
  directory named like a text file, and an unrelated CSV,
  and asserts the three items load in sorted order. It
  then adds a same stem file and asserts the refusal.
