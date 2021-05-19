# globmatch
Exploration of various glob-equivalence algorithms

For the purposes of this repo, glob expressions are expressions like `**/a/s/d/f` or `a/b/*/d/e`.

A path is made up of many fragments, separated by slashes. A fragment is a string literal, like `a` or `some_long_literal`.

`*` is a wildcard: it will match any single fragment between slashes.
`**` is a glob, or recursive wildcard. It will match any number of segments.

This repo explores several algorithms to see if two glob expressions "overlap", i.e., whether there is some path that matches both expressions.


Sample benchmark runs
```
test tests::glob_glob_dp              ... bench:         162 ns/iter (+/- 19)
test tests::glob_glob_optimized       ... bench:         297 ns/iter (+/- 30)
test tests::glob_glob_recursive       ... bench:      19,984 ns/iter (+/- 2,385)
test tests::literal_literal_dp        ... bench:       5,813 ns/iter (+/- 625)
test tests::literal_literal_optimized ... bench:       5,398 ns/iter (+/- 644)
test tests::literal_literal_recursive ... bench:         159 ns/iter (+/- 12)
```
