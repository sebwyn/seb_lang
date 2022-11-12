use std::{fs, error::Error};

use crate::ast_rules::AstRules;

pub struct Token {
    key_name: String,
    value: String,
    line_nr: u32,
    column: u32,
}

pub fn lex(file: &str, rules: &AstRules) -> Result<Vec<Token>, Box<dyn Error>> {
    let source = fs::read_to_string(file)?;
    for line in source.lines() {
        //start by finding all the words in the line, basic tokenizing
        let words = line.split(' ').filter(|s| s.len() > 0).collect::<Vec<&str>>();
        println!("{:?}", words);
        
    }


    Ok(Vec::new())
}