use std::{collections::HashMap, fs, io, iter::Peekable};

use itertools::Itertools;
use serde_json::Value;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=keywords.json");

    let keywords: Value = fs::read_to_string("keywords.json")?.parse()?;
    let mut abbreviations = HashMap::new();
    let mut parse_artifacts = HashMap::new();
    for (key, value) in keywords.as_object().unwrap() {
        let value = value.as_array().unwrap();

        if !value[0].as_bool().unwrap() {
            abbreviations.insert(
                key.as_str(),
                value[1..]
                    .iter()
                    .map(|x| x.as_str().unwrap())
                    .collect::<Vec<_>>(),
            );
        }
    }
    for (key, value) in keywords.as_object().unwrap() {
        let value = value.as_array().unwrap();

        if value[0].as_bool().unwrap() {
            if value[1].is_array() {
                for i in &value[1..] {
                    let i = i.as_array().unwrap();
                    create_artifacts(key, &i[..], &abbreviations, &mut parse_artifacts);
                }
            } else {
                create_artifacts(key, &value[1..], &abbreviations, &mut parse_artifacts);
                eprintln!("{parse_artifacts:?}");
            }
        }
    }

    let mut s = parse_artifacts.into_iter().collect::<Vec<_>>();

    s.iter_mut().for_each(|(_, y)| {
        y.sort_by_key(|x| x.len());
        y.reverse();
    });
    let mut str = String::new();

    str += "use std::sync::LazyLock;\n\nuse ptrie::Trie;\n\n#[derive(Debug, Clone, Copy)]\npub enum IntToken {";
    for (k, _) in &s {
        str += "\n\t";
        let pascal = kebab_to_pascal_case(k);
        str += &pascal;
        str += ",";
    }
    str += "\n}";

    str += "\n\nstatic TRIE: LazyLock<Trie<u8, (IntToken, usize)>> = LazyLock::new(|| {
\tlet mut trie = Trie::new();
";

    for (k, v) in &s {
        for i in v {
            str += "\ttrie.insert(\"";
            str += i;
            str += "\".bytes(), (IntToken::";
            let pascal = kebab_to_pascal_case(k);
            str += &pascal;
            str += ", ";
            str += &i.len().to_string();
            str += "));\n";
        }
    }

    str += "\ttrie
});";

    str += "\n\npub fn make_token(src: &str) -> Option<(IntToken, usize)> {
\tTRIE.find_longest_prefix(src.bytes()).copied()";

    str += "\n}";

    fs::write("src/keywords.rs", str)?;

    Ok(())
}

fn kebab_to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

struct Intersperse<I: Iterator>
where
    I::Item: Clone,
{
    inner: Peekable<I>,
    give_item: bool,
    item: I::Item,
}

impl<I: Iterator> Intersperse<I>
where
    I::Item: Clone,
{
    fn new(iter: I, item: I::Item) -> Self {
        Intersperse {
            inner: iter.peekable(),
            give_item: false,
            item,
        }
    }
}

impl<I: Iterator> Iterator for Intersperse<I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.give_item && self.inner.peek().is_some() {
            self.give_item = false;
            Some(self.item.clone())
        } else {
            self.give_item = true;
            self.inner.next()
        }
    }
}

fn create_artifacts(
    key: &str,
    value: &[Value],
    abbreviations: &HashMap<&str, Vec<&str>>,
    parse_artifacts: &mut HashMap<String, Vec<String>>,
) {
    eprintln!("key: {key}");
    eprintln!("value: {value:?}");
    eprintln!("abbreviations: {abbreviations:?}");
    eprintln!("parse_artifacts: {parse_artifacts:?}");

    let strings = value
        .iter()
        .map(|i| {
            let i = i.as_str().unwrap();
            if i.starts_with("$") {
                let i = i.trim_start_matches("$");
                eprintln!("i: {i}");
                abbreviations.get(i).unwrap().clone()
            } else {
                eprintln!("i: {i}");
                vec![i]
            }
        })
        .collect::<Vec<_>>();
    eprintln!("strings: {strings:?}");
    let delimiters = ["", " ", "_", "-"];
    let artifacts = Intersperse::new(strings.iter().map(|x| x.iter()), delimiters.iter())
        .multi_cartesian_product()
        .map(|x| {
            let mut str = String::new();
            for part in x {
                str += part;
            }
            str
        })
        .collect::<Vec<_>>();
    if let Some(s) = parse_artifacts.get_mut(key) {
        s.extend(artifacts);
    } else {
        parse_artifacts.insert(key.to_string(), artifacts);
    };
}
