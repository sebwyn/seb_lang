use regex::Regex;
use std::{collections::HashMap, error::Error};

use crate::parser_structs::{ArrayType, RuleMap, Sym, SymTree, Kind};

const NAME: &str = r"\$?(?:([a-z|_]+))";
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

        return Some((name, object));
    }
    None
}

fn consume_sym<'a>(source: &'a str) -> (&'a str, Option<Sym>) {
    let (s, sym) = consume_token(source);
    if let Some(sym) = sym {
        return (s, Some(sym));
    }
    let (s, sym) = consume_var(source);
    if let Some(sym) = sym {
        return (s, Some(sym));
    }

    (source, None)
}

//will literally move this string pointer forward if we find our shit
fn consume_token<'a>(source: &'a str) -> (&'a str, Option<Sym>) {
    let non_object = Regex::new(r"^\s*(?:([^\s]+))").unwrap();
    if let Some(capture) = non_object.captures(source) {
        let end = capture.get(1).unwrap().end();
        let name = capture.get(1).unwrap().as_str();

        let sym = match name.chars().nth(0).unwrap() {
            '$' => None,
            '!' => None,
            '%' => None,
            _ => Some(Sym::Token(String::from(name))),
        };

        if sym.is_some() {
            //advance our source and return what we found
            return (source.split_at(end).1, sym);
        }
    }

    (source, None)
}

fn consume_var<'a>(source: &'a str) -> (&'a str, Option<Sym>) {
    let var = Regex::new(r"^\s*\$(?:([^\s^\(]+))").unwrap();

    if let Some(caps) = var.captures(source) {
        let name = caps.get(1).unwrap();
        let (remaining, kind) = consume_inner(source.split_at(name.end()).1, '(', ')');

        let name = String::from(name.as_str());
        let kind = match kind {
            Some(m) => consume_type(m).1.unwrap_or(Kind::UnknownSym(name.clone())),
            None => Kind::UnknownSym(name.clone())
        };

        return (
            remaining,
            Some(Sym::Var((name.clone(), kind)))
        );
    }

    (source, None)
}

pub fn consume_type<'a>(text: &'a str) -> (&'a str, Option<Kind>) {
    //handle arrays in a verbose way
    let type_decorator = Regex::new(r"^\s*%").unwrap();
    if let Some(capture) = type_decorator.captures(text) {
        let decorator = capture.get(0).unwrap();
        let (remaining, inner) = consume_inner(text.split_at(decorator.end()).1, '[', ']');

        if let Some(inner) = inner {
            let (inner_remaining, sym) = consume_sym(inner);
            if let Some(sym) = sym {
                let (_, separator) = consume_token(inner_remaining);
                if let Some(Sym::Token(separator)) = separator {
                    let kind = match sym {
                        Sym::Token(_) => panic!("Expected a type or sym for array def!"),
                        Sym::Var((_, Kind::UnknownSym(a))) => Kind::UnknownSym(a),
                        _ => panic!("Expected a symbol to not have a defined type if used in array def!"),
                    };
                    let array = ArrayType {
                        kind: Box::new(kind),
                        separator,
                    };
                    return (remaining, Some(Kind::Array(array)));
                } else {
                    panic!("Expected a separator token in: {}", text);
                }
            }
            
            println!("Failed to consume sym in {}", inner);

            let (inner_remaining, kind) = consume_type(inner);
            if let Some(kind) = kind {
                let (_, separator) = consume_token(inner_remaining);
                if let Some(Sym::Token(separator)) = separator {
                    let array = ArrayType {
                        kind: Box::new(kind),
                        separator,
                    };
                    return (remaining, Some(Kind::Array(array)));
                }
            }
        }
    }

    let builtin = Regex::new(r"^\s*%(?:([^\s]+))").unwrap();
    if let Some(capture) = builtin.captures(text) {
        let m = capture.get(1).unwrap();
        let remaining = text.split_at(capture.get(0).unwrap().end()).1;

        let name = m.as_str();
        if name.eq("int") {
            return (remaining, Some(Kind::Int))
        } else {
            panic!("Unkown builting type: {}", name);
        }
    }

    let reg = Regex::new(r"^\s*!(?:([^\s]+))").unwrap();
    if let Some(capture) = reg.captures(text) {
        let characters = capture.get(1).unwrap();
        let remaining = text.split_at(capture.get(0).unwrap().end()).1;

        let reg = Regex::new(characters.as_str()).expect(&format!(
            "Failed to parse regex type: {:?}",
            characters.as_str()
        ));

        return (remaining, Some(Kind::Reg(reg)))
    }

    (text, None)
}

//consume potentially nested parentheticals
fn consume_inner<'a>(text: &'a str, oc: char, cc: char) -> (&'a str, Option<&'a str>) {
    let mut chars = text.chars();
    let first_char = chars.next();
    if let Some(first_char) = first_char {
        if first_char == oc {
            let mut index = 0;
            let mut depth = 1;
            for c in chars {
                if c == oc {
                    depth += 1
                } else if c == cc {
                    depth -= 1;
                }
                index += 1;

                if depth == 0 {
                    return (text.split_at(index + 1).1, Some(text.split_at(1).1.split_at(index - 1).0));
                }
            }
        }
    }

    (text, None)
}   

pub fn parse(source: &str) -> Result<RuleMap, Box<dyn Error>> {
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

    Ok(rules)
}
