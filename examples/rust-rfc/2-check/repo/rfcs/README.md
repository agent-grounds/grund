# Skerry RFCs

A change to skerry's public API starts as an RFC.

1. Copy the template,
   [§RFC-0000-my-feature](0000-template.md#rfc-0000-my-feature-the-feature-in-a-few-words)
   in `0000-template.md`, to `text/0000-my-feature.md`, where `my-feature` is
   descriptive, and give its heading the same slug:
   `RFC-0000-my-feature` becomes `RFC-0000-retry-budget`. Leave the number at
   `0000`: it is the pull request's number, which does not exist yet.
2. Open the pull request. Rename the file and the heading's ID to its number,
   and update the RFC PR link.
3. Once the RFC is merged, it is accepted and may be implemented. Code and tests
   cite the reference-level sections they implement.

An accepted RFC changes only by amendment: a later RFC, noted in the
`Amendments` chapter of the RFC it amends.
