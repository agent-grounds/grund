# FS-001-wraps: Wrapped citation examples

Lead prose flattens [§FS-002-target.1](FS-002-target.md#1-detail).

## 1. Both fence characters

Before the fences, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.

```markdown
[§FS-002-target.1](FS-002-target.md#1-detail)
[§FS-<foo>.3.1](<relative-path>#<anchor>)
```

Between the fences, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.

~~~markdown
[§FS-002-target.1](FS-002-target.md#1-detail)
[§FS-<foo>.3.1](<relative-path>#<anchor>)
~~~

After the fences, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.

## 2. Close and resume

Before the fence, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.

````markdown
[§FS-002-target.1](FS-002-target.md#1-detail)
~~~
[§FS-002-target.1](FS-002-target.md#1-detail)
```
[§FS-002-target.1](FS-002-target.md#1-detail)
`````

After the valid longer closer, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.

## 3. Unclosed fence

Before the fence, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.

~~~~markdown
[§FS-002-target.1](FS-002-target.md#1-detail)
~~~
[§FS-002-target.1](FS-002-target.md#1-detail)
