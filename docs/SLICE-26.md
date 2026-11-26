# Slice 26. A security note

The repository carried no security note, and the tool's
whole posture, local runs over files the user holds, was
stated only in passing in the design record.

## What landed

- SECURITY.md at the root states the posture plainly, no
  network calls in the counting path, no telemetry, no
  credentials held, and no promise in version 1 to detect
  or redact personal information in item text, with
  custody of the input files named as the control.
- It gives the reporting route and states there is no
  bug bounty and no hosted service.
- The changelog records it, and the documents index
  gains its slice 26 line.
