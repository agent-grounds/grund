# RFC-0007-retry-policy: Retry policy

## summary: Summary

Retry an idempotent request at most three times.

## motivation: Motivation

A transient failure should not fail the caller.

## guide-level-explanation: Guide-level Explanation

Each retry waits twice as long as the one before it.
