# RFC-retry: Retry policy

## summary: Summary

Retry a failed request with capped exponential backoff.

## unresolved-questions: Unresolved questions

Two questions are still open; each is a chapter titled Open.

### unresolved-questions.cap: Open

Should the cap be configurable per call?

### unresolved-questions.budget: Open

Should retries share one budget across calls?
