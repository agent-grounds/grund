# RFC-0007-retry-policy: Retry policy

## summary: Summary

Retry an idempotent request three times.

## motivation: Motivation

A dropped connection should not fail a payout.

## guide-level-explanation: Guide-level explanation

The client retries with backoff.
