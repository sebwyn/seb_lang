use std::{collections::HashMap, fs, error::Error};

use crate::ast_rules::{AstRules, RuleNode};

struct Cluster<'a, T> {
    prefix: String,
    index: u32,

    nodes: String,
    rules: String,
    links: String,

    tokens: &'a Vec<(String, T)>,
}

impl<T> Cluster<'_, T> {
    fn new<'a>(name: &str, node: &RuleNode, tokens: &'a Vec<(String, T)>) -> String {
        let mut cluster = Cluster {
            prefix: String::from(name),
            index: 0,
            nodes: String::new(),
            rules: String::new(),
            links: String::new(),
            tokens,
        };
        //create a node without prefixing formatting and shit

        cluster.nodes += &format!("\t\t{} [style = filled; color = red;]\n", name);
        cluster.index += 1;

        cluster.dump_node(&name, node);

        let mut cluster_text = format!("\tsubgraph cluster_{} {{\n", name);
        cluster_text += &cluster.nodes;
        cluster_text += &cluster.rules;

        cluster_text += "\t\t}\n";

        cluster_text += &cluster.links;

        cluster_text
    }

    fn create_node(&mut self, name: &str, node: &RuleNode) -> String {
        let dot_name = format!("{}_{}", self.prefix, self.index);
        let shape = if node.terminal { "box" } else { "ellipse" };

        self.nodes += &format!("\t\t{} [label = \"{}\"; style = filled; shape = \"{}\"]\n", dot_name, name, shape);
        self.index += 1;

        //if the name is not a token, also add the node to the linker
        if self.tokens.iter().find(|(key, definition)| key.as_str() == name).is_none() {
            //add this to the linker
            self.links += &format!("\t{} -> {}\n", dot_name, name);
        }

        dot_name
    }

    fn dump_node(&mut self, dot_node_name: &str, node: &RuleNode) {
        for (child_name, child) in node.children.iter() {
            let child_dot_name = self.create_node(child_name, child);

            self.rules += &format!("\t\t{} -> {}\n", dot_node_name, child_dot_name);
            self.dump_node(&child_dot_name, child);
        }
    }
}

pub fn print_ast(file: &str, ast: &AstRules) -> Result<(), Box<dyn Error>> {
    //only do the first one
    let mut dot_out = String::from("digraph AST {\n");
    for (name, rule) in ast.rules.iter() {
        dot_out += &Cluster::new(name, rule, &ast.tokens);
    }
    dot_out += "}";
    fs::write(file, dot_out)?;

    Ok(())
}