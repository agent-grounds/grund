# AR index

Every `AR` declaration, one linked entry each.

- [§AR-c](../../src/router.h)
- [§AR-clojure](../../src/router.clj)
- [§AR-cpp](../../src/router.hpp)
- [§AR-csharp](../../src/Router.cs)
- [§AR-dart](../../src/router.dart)
- [§AR-elisp](../../src/router.el)
- [§AR-go](../../src/router.go)
- [§AR-haskell](../../src/Router.hs)
- [§AR-haskell-block](../../src/BlockRouter.hs)
- [§AR-java](../../src/Router.java)
- [§AR-java-markdown](../../src/MarkdownRouter.java)
- [§AR-javascript](../../src/router.js)
- [§AR-kotlin](../../src/Router.kt)
- [§AR-lua](../../src/router.lua)
- [§AR-objc](../../src/Router.m)
- [§AR-php](../../src/Router.php)
- [§AR-python](../../src/router.py)
- [§AR-ruby](../../src/router.rb)
- [§AR-rust](../../src/router.rs)
- [§AR-scala](../../src/Router.scala)
- [§AR-shell](../../src/router.sh)
- [§AR-sql](../../src/router.sql)
- [§AR-swift](../../src/Router.swift)
- [§AR-typescript](../../src/router.ts)

The golden pins the verdicts of grund at the time it was written, known defects
included. Each `unknown reference AR-lua`, `AR-haskell`, `AR-haskell-block`,
`AR-clojure` and `AR-elisp` line is grund#703: those doc forms host no
declaration yet. Each `unknown project alias api` line is grund#702: the
interior of a multi-line string is read as prose. Those fixes remove the lines
on refresh, leaving one `unknown reference FS-gone-<lang>` per language.
`grund.toml` lists every extension explicitly, so the case does not settle
grund#705, the default extension list.
