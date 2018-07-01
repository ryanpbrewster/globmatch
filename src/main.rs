use std::collections::BTreeMap;
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

fn main() -> Result<(), Box<Error>> {
    println!("Hello, world!");
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let path: Path = line?.parse()?;
        println!("{:?}", path);
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
                other => Err("only * and ** fragments may contain *"),
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
impl Path {
    fn new() -> Path {
        Path(Vec::new())
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
