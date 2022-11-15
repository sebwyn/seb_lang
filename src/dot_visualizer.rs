use std::{fs, error::Error, collections::HashMap};

use crate::parser_structs::{SymTree, SymNode, Sym, RuleMap};

struct Cluster {
    prefix: String,
    index: u32,

    nodes: String,
    rules: String,
    links: String,
}

impl Cluster {
    fn new<'a>(name: &str, tree: &SymTree) -> String {
        let mut cluster = Cluster {
            prefix: String::from(name),
            index: 0,
            nodes: String::new(),
            rules: String::new(),
            links: String::new()
        };
        //create a node without prefixing formatting and shit

        cluster.nodes += &format!("\t\t{} [style = filled; color = red;]\n", name);
        cluster.index += 1;

        cluster.dump_node(&name, &tree.children);

        let mut cluster_text = format!("\tsubgraph cluster_{} {{\n", name);
        cluster_text += &cluster.nodes;
        cluster_text += &cluster.rules;

        cluster_text += "\t}\n";

        cluster_text += &cluster.links;

        cluster_text
    }

    fn create_node(&mut self, node: &SymNode, priority: usize) -> String {
        let dot_name = format!("{}_{}", self.prefix, self.index);
        self.index += 1;
        //let shape = if node.terminal { "box" } else { "ellipse" };
        let shape = "ellipse";

        let label = match &node.sym {
            crate::parser_structs::Sym::Token(s) => String::from(s),
            crate::parser_structs::Sym::Var((name, _)) => name.clone(),
        };

        self.nodes += &format!("\t\t{} [label = \"{}: {}\"; style = filled; shape = \"{}\"]\n", dot_name, label, priority, shape);

        if let Sym::Var((name, kind)) = &node.sym {
            self.links += &format!("\t{} -> {}\n", dot_name, name);
        }

        dot_name
    }

    fn dump_node(&mut self, dot_node_name: &str, nodes: &[SymNode]) {
        for (priority, node) in nodes.iter().enumerate() {
            let child_dot_name = self.create_node(node, priority);

            self.rules += &format!("\t\t{} -> {}\n", dot_node_name, child_dot_name);
            self.dump_node(&child_dot_name, &node.children);
        }
    }
}

pub fn print_sym_tree(file: &str, ast: &RuleMap) -> Result<(), Box<dyn Error>> {
    //only do the first one
    let mut dot_out = String::from("digraph AST {\n");
    for (name, tree) in ast.iter() {
        dot_out += &Cluster::new(name, tree);
    }
    dot_out += "}";
    fs::write(file, dot_out)?;

    Ok(())
}