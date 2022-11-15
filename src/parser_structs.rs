use regex::Regex;

use std::collections::HashMap;

pub type RuleMap = HashMap<String, SymTree>;

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub kind: Box<Kind>,
    pub separator: String,
}

impl PartialEq for ArrayType {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.separator == other.separator
    }
}

//an object is a kind of variable, and its kind of builtin, but its not atomic
#[derive(Debug, Clone)]
pub enum Kind {
    Reg(Regex),
    Array(ArrayType),
    Int,
    UnknownSym(String)
}

impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Reg(l0), Self::Reg(r0)) => l0.as_str() == r0.as_str(),
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::UnknownSym(l0), Self::UnknownSym(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Sym {
    Token(String), //just a token
    Var((String, Kind)), //has an underlying type
}

impl PartialEq for Sym {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Token(l0), Self::Token(r0)) => l0 == r0,
            (Self::Var(l0), Self::Var(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Debug)]
pub struct SymTree {
    pub children: Vec<SymNode>
}

impl SymTree {
    pub fn new() -> Self {
        Self {
            children: Vec::new()
        }
    }

    pub fn add_path(&mut self, path: &[Sym]) {
        if let Some(sym) = path.get(0) {
            //find the path in what we already have
            if let Some(child) = self.children.iter_mut().find(|c| c.sym == *sym) {
                child.add_path(path.split_at(1).1);
            } else {
                //create a child, and add the path
                self.children.push(SymNode::new(sym.clone()));
                let last_index = self.children.len()-1;
                let last = self.children.get_mut(last_index).unwrap();
                last.add_path(path.split_at(1).1);
            }
        }
    }
}

//this will contain the rules for a single variable
#[derive(Debug)]
pub struct SymNode {
    pub sym: Sym,
    pub children: Vec<SymNode>,
}
impl SymNode {
    fn new(sym: Sym) -> Self {
        Self {
            sym,
            children: Vec::new(),
        }
    }

    fn add_path(&mut self, path: &[Sym]) {
        if let Some(sym) = path.get(0) {
            //find the path in what we already have
            if let Some(child) = self.children.iter_mut().find(|c| c.sym == *sym) {
                child.add_path(path.split_at(1).1);
            } else {
                //create a child, and add the path
                self.children.push(SymNode::new(sym.clone()));
                let last_index = self.children.len()-1;
                let last = self.children.get_mut(last_index).unwrap();
                last.add_path(path.split_at(1).1);
            }
        }
    }
}