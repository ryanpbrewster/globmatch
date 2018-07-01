use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

fn main() -> Result<(), Box<Error>> {
    println!("Hello, world!");
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
                println!("OVERLAP:");
                println!("  - {:?}", paths[i]);
                println!("  - {:?}", paths[j]);
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
impl FromStr for Path {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
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

    fn overlap(a: &[Fragment], b: &[Fragment]) -> bool {
        println!("checking {:?}, {:?}", a.first(), b.first());
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
                Path::overlap(&a[1..], &b[1..])
            }
            // Normal recursion
            (Some(&Fragment::Literal(ref a0)), Some(&Fragment::Literal(ref b0))) => {
                a0 == b0 && Path::overlap(&a[1..], &b[1..])
            }
            // Glob handling
            (Some(&Fragment::Glob), None) => Path::overlap(&a[1..], b),
            (None, Some(&Fragment::Glob)) => Path::overlap(a, &b[1..]),
            // Left glob
            (Some(&Fragment::Glob), Some(&Fragment::Literal(_)))
            | (Some(&Fragment::Glob), Some(&Fragment::Wildcard)) => {
                Path::overlap(&a[1..], &b[1..]) || Path::overlap(a, &b[1..])
            }
            // Right glob
            (Some(&Fragment::Literal(_)), Some(&Fragment::Glob))
            | (Some(&Fragment::Wildcard), Some(&Fragment::Glob)) => {
                Path::overlap(&a[1..], &b[1..]) || Path::overlap(&a[1..], b)
            }
            // Both glob
            (Some(&Fragment::Glob), Some(&Fragment::Glob)) => {
                Path::overlap(&a[1..], &b[1..])
                    || Path::overlap(a, &b[1..])
                    || Path::overlap(&a[1..], b)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_paths_overlap() {
        assert!(Path::overlap(Path::new().as_ref(), Path::new().as_ref()));
    }

    #[test]
    fn literal_paths_overlap_if_equal() {
        assert!(Path::overlap(
            "a/b/c".parse::<Path>().unwrap().as_ref(),
            "a/b/c".parse::<Path>().unwrap().as_ref()
        ));
        assert!(!Path::overlap(
            "a/b/c".parse::<Path>().unwrap().as_ref(),
            "d/e/f".parse::<Path>().unwrap().as_ref()
        ));
    }

    #[test]
    fn wildcards_match_any_literal() {
        assert!(Path::overlap(
            "a/b/*".parse::<Path>().unwrap().as_ref(),
            "a/b/c".parse::<Path>().unwrap().as_ref()
        ));
    }
    #[test]
    fn wildcards_do_not_match_multiple_literals() {
        assert!(!Path::overlap(
            "a/b/*".parse::<Path>().unwrap().as_ref(),
            "a".parse::<Path>().unwrap().as_ref()
        ));
        assert!(!Path::overlap(
            "a/b/*".parse::<Path>().unwrap().as_ref(),
            "a/b/c/d/e".parse::<Path>().unwrap().as_ref()
        ));
    }

    #[test]
    fn globs_match_empty_paths() {
        assert!(Path::overlap(
            "**".parse::<Path>().unwrap().as_ref(),
            Path::new().as_ref()
        ));
    }

    #[test]
    fn globs_match_many_literals() {
        assert!(Path::overlap(
            "**".parse::<Path>().unwrap().as_ref(),
            "a/b/c/d/e".parse::<Path>().unwrap().as_ref()
        ));
    }

    #[test]
    fn globs_with_suffixes_only_match_suffixed_strings() {
        assert!(Path::overlap(
            "**/d/e".parse::<Path>().unwrap().as_ref(),
            "a/b/c/d/e".parse::<Path>().unwrap().as_ref()
        ));
        assert!(!Path::overlap(
            "**/d/e".parse::<Path>().unwrap().as_ref(),
            "a/b/c/d".parse::<Path>().unwrap().as_ref()
        ));
        assert!(!Path::overlap(
            "**/d/e".parse::<Path>().unwrap().as_ref(),
            "a/b/c/d/e/f".parse::<Path>().unwrap().as_ref()
        ));
    }
}
