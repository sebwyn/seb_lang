use std::collections::HashMap;

use regex::Regex;

use crate::parser_structs::{RuleMap, Sym, SymNode};

enum Value {
    String(String),
    Int(i32),
    IntArray(Vec<i32>),
    StringArray(Vec<String>),
    Object(Box<Object>),

    //intermediate
    KeyVal((String, Box<Value>))
}

struct Object {
    name: String,
    keys: HashMap<String, Value>,
}

fn consume_array<'a>(stream: &'a str, kind: &Box<Sym>, sep: &str) -> Option<(&'a str, Value)> {
    None
}

fn consume_token<'a>(stream: &'a str, token: &str) -> Option<&'a str> {
    None
}

fn consume_reg<'a>(stream: &'a str, reg: &Regex) -> Option<(&'a str, Value)> {
    None
}

fn consume_int<'a>(stream: &'a str) -> Option<(&'a str, Value)> {
    None
}

fn parse_node<'a>(text: &str, node: &SymNode, rules: &RuleMap) -> Option<(&'a str, Option<(String, Value)>)> {
    match &node.sym {
        Sym::Token(s) => {
            //consume_token(text, s).and_then(|rem| Some((rem, None)))
            None
        },
        Sym::Var(name) => {
            //parse_with_rule(text, name, rules).and_then(|| )
            None
        },
    }
}

fn parse_children<'a>(text: &str, object: &mut Object, children: &[SymNode], rules: &RuleMap) -> Option<(&'a str, Option<Value>)> {
    for node in children {
        let mut text = text;

        //try to match this child node
        if let Some((remaining, value)) = parse_node(text, node, rules) {

        }

        //parse children of this node, with the object w
    }

    None
}

//for now by default parse every rule, into an object
fn parse_with_rule<'a>(text: &str, rule: &str, rules: &RuleMap) -> Option<(&'a str, Object)> {
    let rule = rules.get(rule).expect(&format!("Parsing with undefined rule: {}!", rule));
    None
}