# globmatch
Exploration of various glob-equivalence algorithms

For the purposes of this repo, glob expressions are expressions like `**/a/s/d/f` or `a/b/*/d/e`.

A path is made up of many fragments, separated by slashes. A fragment is a string literal, like `a` or `some_long_literal`.

`*` is a wildcard: it will match any single fragment between slashes.
`**` is a glob, or recursive wildcard. It will match any number of segments.

This repo explores several algorithms to see if two glob expressions "overlap", i.e., whether there is some path that matches both expressions.
