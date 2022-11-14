use regex::Regex;

#[derive(Debug, Clone)]
pub enum Sym {
    //non data objects
    Token(String), //just a token
    Reg(Regex),    //a regex match, potentially mapped to a type

    //data objects
    Var(String), //could be a rule, could be a token its used to construct data

    //this is a functional search unit, where it will keep searching for tokens
    //using the last item as the separator %[ $expr , ]
    Array((Box<Sym>, String)),

    Int,
}

impl PartialEq for Sym {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Token(l0), Self::Token(r0)) => l0 == r0,
            (Self::Reg(l0), Self::Reg(r0)) => l0.as_str() == r0.as_str(),
            (Self::Var(l0), Self::Var(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
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