# FS-skip: Skipped level

## format: Formatting

#### .1: Deep

Skipped.

# FS-orphan: Orphan parent

## format: Formatting

### missing.x: Orphan

#### .1: Child

Under an orphan.

# FS-dup-before: Duplicate parent before

## format: Formatting

### format.1: One

### format.1: Two

#### .1: Child

Under the second claimant.

# FS-dup-after: Duplicate parent after

## format: Formatting

### format.1: One

#### .1: Child

Under the first claimant.

### format.1: Two

# FS-fence: Parent in a fence

## format: Formatting

```markdown
### format.1: Fenced
```

#### .1: Child

The would-be parent is fenced.

# FS-plain: Plain parent

## format: Formatting

### Plain

#### .1: Child

Under a plain heading.

# FS-chain: Under a failed relative heading

## format: Formatting

#### .1: Deep

##### .2: Deeper

Both fail.

# FS-mismatch: A parent with a depth mismatch

## format: Formatting

#### format.1: Too deep

##### .1: Child

The parent carries its own finding and the child has none.
