#![feature(test)]
extern crate test;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::str::FromStr;

fn main() -> Result<(), Box<Error>> {
    let paths: Vec<Path> = {
        let stdin = std::io::stdin();
        let mut buf = Vec::new();
        for line in stdin.lock().lines() {
            buf.push(line?.parse()?)
        }
        buf
    };
    for i in 1..paths.len() {
        for j in 0..i {
            if Path::overlap(paths[i].as_ref(), &paths[j].as_ref()) {
                println!("{} overlaps with {}", paths[i], paths[j]);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub enum Fragment {
    Literal(String),
    Wildcard,
    Glob,
}

impl Display for Fragment {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(match self {
            Fragment::Literal(s) => s,
            Fragment::Wildcard => "*",
            Fragment::Glob => "**",
        })
    }
}
impl FromStr for Fragment {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Fragment, Self::Err> {
        if s.is_empty() {
            return Err("no empty fragments");
        }
        if s.contains('*') {
            return match s {
                "*" => Ok(Fragment::Wildcard),
                "**" => Ok(Fragment::Glob),
                _other => Err("only * and ** fragments may contain *"),
            };
        }
        Ok(Fragment::Literal(s.to_owned()))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Path(Vec<Fragment>);
impl Display for Path {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for fragment in &self.0 {
            write!(f, "/{}", fragment)?;
        }
        Ok(())
    }
}
impl FromStr for Path {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.is_empty() {
            return Ok(Path::new());
        }
        let fragments = s
            .split('/')
            .map(|t| t.parse::<Fragment>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Path(fragments))
    }
}
impl IntoIterator for Path {
    type Item = Fragment;
    type IntoIter = std::vec::IntoIter<Fragment>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}
impl AsRef<[Fragment]> for Path {
    fn as_ref(&self) -> &[Fragment] {
        &self.0
    }
}

impl Path {
    fn new() -> Path {
        Path(Vec::new())
    }

    /**
     * Recursive implementation of the glob match algorithm.
     * Good best-case behavior, very bad worst-case behavior,
     */
    fn recursive_overlap(a: &[Fragment], b: &[Fragment]) -> bool {
        match (a.first(), b.first()) {
            // Trivial success
            (None, None) => true,
            // Trivial failures
            (None, Some(&Fragment::Literal(_)))
            | (None, Some(&Fragment::Wildcard))
            | (Some(&Fragment::Literal(_)), None)
            | (Some(&Fragment::Wildcard), None) => false,
            // Easy recursion
            (Some(&Fragment::Literal(_)), Some(&Fragment::Wildcard))
            | (Some(&Fragment::Wildcard), Some(&Fragment::Literal(_)))
            | (Some(&Fragment::Wildcard), Some(&Fragment::Wildcard)) => {
                Path::recursive_overlap(&a[1..], &b[1..])
            }
            // Normal recursion
            (Some(&Fragment::Literal(ref a0)), Some(&Fragment::Literal(ref b0))) => {
                a0 == b0 && Path::recursive_overlap(&a[1..], &b[1..])
            }
            // Glob handling
            (Some(&Fragment::Glob), None) => Path::recursive_overlap(&a[1..], b),
            (None, Some(&Fragment::Glob)) => Path::recursive_overlap(a, &b[1..]),
            // Left glob
            (Some(&Fragment::Glob), Some(&Fragment::Literal(_)))
            | (Some(&Fragment::Glob), Some(&Fragment::Wildcard)) => {
                Path::recursive_overlap(&a[1..], &b[1..]) || Path::recursive_overlap(a, &b[1..])
            }
            // Right glob
            (Some(&Fragment::Literal(_)), Some(&Fragment::Glob))
            | (Some(&Fragment::Wildcard), Some(&Fragment::Glob)) => {
                Path::recursive_overlap(&a[1..], &b[1..]) || Path::recursive_overlap(&a[1..], b)
            }
            // Both glob
            (Some(&Fragment::Glob), Some(&Fragment::Glob)) => {
                Path::recursive_overlap(&a[1..], &b[1..])
                    || Path::recursive_overlap(a, &b[1..])
                    || Path::recursive_overlap(&a[1..], b)
            }
        }
    }

    /**
     * Dynamic-programming implementation of the glob match algorithm.
     * Acceptable best-case behavior, acceptable worst-case behavior,
     */
    fn dp_overlap(a: &[Fragment], b: &[Fragment]) -> bool {
        let (m, n) = (a.len() + 1, b.len() + 1);
        let mut memo: Vec<bool> = vec![false; m * n];
        memo[0] = true;
        for i in 1..m {
            memo[i * n] = memo[(i - 1) * n] && a[i - 1] == Fragment::Glob;
        }
        for j in 1..n {
            memo[j] = memo[j - 1] && b[j - 1] == Fragment::Glob;
        }
        for i in 1..m {
            for j in 1..n {
                memo[i * n + j] = match (&a[i - 1], &b[j - 1]) {
                    // Literals
                    (Fragment::Literal(ref a0), Fragment::Literal(ref b0)) => {
                        a0 == b0 && memo[(i - 1) * n + (j - 1)]
                    }
                    // Wildcards
                    (Fragment::Literal(_), Fragment::Wildcard)
                    | (Fragment::Wildcard, Fragment::Literal(_))
                    | (Fragment::Wildcard, Fragment::Wildcard) => memo[(i - 1) * n + (j - 1)],
                    // Left glob
                    (Fragment::Glob, Fragment::Literal(_))
                    | (Fragment::Glob, Fragment::Wildcard) => {
                        memo[i * n + (j - 1)] || memo[(i - 1) * n + (j - 1)]
                    }
                    // Right glob
                    (Fragment::Literal(_), Fragment::Glob)
                    | (Fragment::Wildcard, Fragment::Glob) => {
                        memo[(i - 1) * n + j] || memo[(i - 1) * n + (j - 1)]
                    }
                    // Both glob
                    (Fragment::Glob, Fragment::Glob) => {
                        memo[(i - 1) * n + j]
                            || memo[i * n + (j - 1)]
                            || memo[(i - 1) * n + (j - 1)]
                    }
                };
            }
        }
        memo[m * n - 1]
    }

    /**
     * Dynamic-programming implementation of the glob match algorithm with lower memory use.
     * Acceptable best-case behavior, acceptable worst-case behavior.
     */
    fn optimized_overlap(a: &[Fragment], b: &[Fragment]) -> bool {
        let (m, n) = (a.len() + 1, b.len() + 1);
        let mut cur: Vec<bool> = vec![false; n];
        cur[0] = true;
        for j in 1..n {
            cur[j] = cur[j - 1] && b[j - 1] == Fragment::Glob;
        }

        let mut prev: Vec<bool> = vec![false; n];
        for i in 1..m {
            std::mem::swap(&mut prev, &mut cur);
            cur[0] = prev[0] && a[i - 1] == Fragment::Glob;
            for j in 1..n {
                cur[j] = match (&a[i - 1], &b[j - 1]) {
                    // Literals
                    (Fragment::Literal(ref a0), Fragment::Literal(ref b0)) => {
                        a0 == b0 && prev[j - 1]
                    }
                    // Wildcards
                    (Fragment::Literal(_), Fragment::Wildcard)
                    | (Fragment::Wildcard, Fragment::Literal(_))
                    | (Fragment::Wildcard, Fragment::Wildcard) => prev[j - 1],
                    // Left glob
                    (Fragment::Glob, Fragment::Literal(_))
                    | (Fragment::Glob, Fragment::Wildcard) => cur[j - 1] || prev[j - 1],
                    // Right glob
                    (Fragment::Literal(_), Fragment::Glob)
                    | (Fragment::Wildcard, Fragment::Glob) => prev[j] || prev[j - 1],
                    // Both glob
                    (Fragment::Glob, Fragment::Glob) => cur[j - 1] || prev[j - 1] || prev[j],
                };
            }
        }
        cur[b.len()]
    }

    fn overlap(a: &[Fragment], b: &[Fragment]) -> bool {
        Path::optimized_overlap(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlap(a: &str, b: &str) -> bool {
        Path::overlap(
            a.parse::<Path>().unwrap().as_ref(),
            b.parse::<Path>().unwrap().as_ref(),
        )
    }

    #[test]
    fn empty_paths_overlap() {
        assert!(overlap("", ""));
    }

    #[test]
    fn literal_paths_overlap_if_equal() {
        assert!(overlap("a/b/c", "a/b/c"));
        assert!(!overlap("a/b/c", "d/e/f"));
    }

    #[test]
    fn wildcards_match_any_literal() {
        assert!(overlap("a/b/*", "a/b/c"));
    }
    #[test]
    fn wildcards_do_not_match_multiple_literals() {
        assert!(!overlap("a/b/*", "a"));
        assert!(!overlap("a/b/*", "a/b/c/d/e"));
    }

    #[test]
    fn globs_match_empty_paths() {
        assert!(overlap("**", ""));
    }

    #[test]
    fn globs_match_many_literals() {
        assert!(overlap("**", "a/b/c/d/e"));
    }

    #[test]
    fn globs_with_suffixes_only_match_suffixed_strings() {
        assert!(overlap("**/d/e", "a/b/c/d/e"));
        assert!(!overlap("**/d/e", "a/b/c/d"));
        assert!(!overlap("**/d/e", "a/b/c/d/e/f"));
    }

    use test::Bencher;
    fn diff(base: &[Fragment]) -> (Path, Path) {
        let p1 = {
            let mut buf = base.to_vec();
            buf.push(Fragment::Literal("a".to_owned()));
            Path(buf)
        };
        let p2 = {
            let mut buf = base.to_vec();
            buf.push(Fragment::Literal("b".to_owned()));
            Path(buf)
        };
        (p1, p2)
    }
    fn pathological_globs() -> (Path, Path) {
        diff(&vec![Fragment::Glob; 5])
    }
    fn pathological_literal() -> (Path, Path) {
        diff(&vec![Fragment::Literal("a".to_owned()); 32])
    }
    #[bench]
    fn glob_glob_recursive(bencher: &mut Bencher) {
        let (p1, p2) = pathological_globs();
        bencher.iter(|| Path::recursive_overlap(p1.as_ref(), p2.as_ref()));
    }
    #[bench]
    fn glob_glob_dp(bencher: &mut Bencher) {
        let (p1, p2) = pathological_globs();
        bencher.iter(|| Path::dp_overlap(p1.as_ref(), p2.as_ref()));
    }
    #[bench]
    fn glob_glob_optimized(bencher: &mut Bencher) {
        let (p1, p2) = pathological_globs();
        bencher.iter(|| Path::optimized_overlap(p1.as_ref(), p2.as_ref()));
    }

    #[bench]
    fn literal_literal_recursive(bencher: &mut Bencher) {
        let (p1, p2) = pathological_literal();
        bencher.iter(|| Path::recursive_overlap(p1.as_ref(), p2.as_ref()));
    }
    #[bench]
    fn literal_literal_dp(bencher: &mut Bencher) {
        let (p1, p2) = pathological_literal();
        bencher.iter(|| Path::dp_overlap(p1.as_ref(), p2.as_ref()));
    }
    #[bench]
    fn literal_literal_optimized(bencher: &mut Bencher) {
        let (p1, p2) = pathological_literal();
        bencher.iter(|| Path::optimized_overlap(p1.as_ref(), p2.as_ref()));
    }
}
