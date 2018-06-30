use std::collections::BTreeMap;
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

fn main() -> Result<(), Box<Error>> {
    println!("Hello, world!");
    let mut trie = Trie::default();
    {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let path: Path = line?.parse()?;
            println!("{:?}", path);
            println!("{:?}", overlaps(&trie, &path));
            trie.insert(path);
            println!("{:?}", trie);
        }
    }
    Ok(())
}

fn overlaps(trie: &Trie, path: &Path) -> Vec<Path> {
    if path.is_empty() && trie.count > 0 {
        return vec![Path::new()];
    }

    Vec::new()
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub enum Fragment {
    Glob,
    Wildcard,
    Literal(String),
}

impl FromStr for Fragment {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Fragment, Self::Err> {
        Ok(match s {
            "*" => Fragment::Wildcard,
            "**" => Fragment::Glob,
            other => Fragment::Literal(other.to_owned()),
        })
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
impl Path {
    fn new() -> Path {
        Path(Vec::new())
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct Trie {
    children: BTreeMap<Fragment, Trie>,
    count: usize,
}

impl Trie {
    fn insert(&mut self, path: Path) {
        let mut cur = path.0.into_iter().fold(self, |cur, fragment| {
            cur.children.entry(fragment).or_insert_with(Trie::default)
        });
        cur.count += 1;
    }
}
