use std::{collections::HashMap, error::Error};
use regex::Regex;

use crate::{Sym, SymTree};

const NAME: &str = r"\$?(?:([a-z]+))";
const SYM: &str = r"\s*(?:([^\s]+))";

fn consume_rule(line: &str) -> Option<(String, Vec<Sym>)> {
    //define some base tokens
    //let name = Regex::new("[a-z]+")?;
    let rule = format!(r"^{}:(?:(.*))$", NAME);
    let rule = Regex::new(rule.as_str()).unwrap();

    if let Some(capture) = rule.captures(line) {
        let name = String::from(capture.get(1)?.as_str());
        let mut syms = capture.get(2)?.as_str();

        let mut object = Vec::new();

        loop {
            let (s, sym) = consume_sym(syms);
            if let Some(sym) = sym {
                syms = s;
                object.push(sym);
            } else {
                break;
            }
        }

        if syms.len() > 0 {
            println!("Maybe something up {}", syms);
        }

        println!("{}: {:?}", name, object);
    }
    None
}

fn consume_sym<'a>(source: &'a str) -> (&'a str, Option<Sym>) {
    let (s, sym) = consume_non_object(source);
    if let Some(sym) = sym {
        return (s, Some(sym));
    }
    let (s, sym) = consume_builtin(source);
    if let Some(sym) = sym {
        return (s, Some(sym));
    }

    (source, None)
}

//will literally move this string pointer forward if we find our shit
fn consume_non_object<'a>(source: &'a str) -> (&'a str, Option<Sym>) {
    let non_object = Regex::new(SYM).unwrap();
    if let Some(capture) = non_object.captures(source) {
        let end = capture.get(1).unwrap().end();
        let name = capture.get(1).unwrap().as_str();

        let sym = match name.chars().nth(0).unwrap() {
            '$' => Some(Sym::Var(String::from(name.split_at(1).1))),
            '!' => Some(Sym::Reg(
                Regex::new(name.split_at(1).1)
                    .expect(&format!("Invalid regex: {}", name.split_at(1).1)),
            )),
            '%' => None, //object parsing is handled somwhere else
            _ => Some(Sym::Token(String::from(name))),
        };

        if sym.is_some() {
            //advance our source and return what we found
            return (source.split_at(end).1, sym);
        }
    }

    (source, None)
}

fn consume_builtin<'a>(source: &'a str) -> (&'a str, Option<Sym>) {
    //let object = Regex::new(r"\s*%\{(?:(.*))\}").unwrap();
    let array = Regex::new(r"\s*%\[(?:(.*))\]").unwrap();
    //let t = Regex::new(r"\s*%[^\s]+").unwrap();

    if let Some(capture) = array.captures(source) {
        let inner = capture.get(1).unwrap();

        let (remaining, sym) = consume_sym(inner.as_str());
        if let Some(sym) = sym {
            let (_, separator) = consume_sym(remaining);
            if let Some(Sym::Token(separator)) = separator {
                return (
                    source.split_at(capture.get(0).unwrap().end()).1,
                    Some(Sym::Array((Box::new(sym), separator))),
                );
            }
        }
    }

    let builtin = Regex::new(r"\s*%(?:([^\s]+))").unwrap();

    if let Some(capture) = builtin.captures(source) {
        let m = capture.get(1).unwrap();
        let remaining = source.split_at(m.end()).1;

        let name = m.as_str();
        if name.eq("int") {
            return (remaining, Some(Sym::Int));
        }
    }

    (source, None)
}

pub fn parse(source: &str) -> Result<String, Box<dyn Error>> {
    let mut rules: HashMap<String, SymTree> = HashMap::new();
    for line in source.lines() {
        if let Some((name, syms)) = consume_rule(line) {
            //get or insert
            let sym_tree = match rules.get_mut(&name) {
                Some(node) => node,
                None => {
                    rules.insert(name.clone(), SymTree::new());
                    rules.get_mut(&name).unwrap()
                }
            };
            sym_tree.add_path(&syms);
        }
    }

    for (name, tree) in rules {
        println!("{}: {:?}", name, tree);
    }

    Ok(String::from(""))
}
