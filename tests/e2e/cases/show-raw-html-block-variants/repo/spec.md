# FS-widget: Check

The lead.

<pre class="one-line">kept on one line</pre>

## 1. After a one-line block

<SCRIPT>
# FS-widget: in a script
</script>

<style>
## 9. In a style
</style>

<textarea
## 9. In a textarea
</TEXTAREA>

<prefix>

## 2. After a tag that only starts like pre

```markdown
<pre>
```

## 3. After a fence holding an unclosed pre

   <pre>
```
## 9. A fence opener inside pre opens nothing
</pre>

## 4. After a pre holding a fence opener

    <pre>

## 5. After a pre indented four spaces
