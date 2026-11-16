---
applies_to: ["client", "server", "aws-sdk-rust"]
authors: ["FalkWoldmann"]
references: ["smithy-rs#3826"]
breaking: false
new_feature: false
bug_fix: false
---
Replace the `time` crate with [`jiff`](https://crates.io/crates/jiff) 1.0 for date-time parsing and formatting in `aws-smithy-types`, `aws-sigv4`, and `aws-config`. This is an internal change: no public API changes, and the supported timestamp formats and their accepted inputs are the same as before.
