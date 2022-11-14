use std::{fs, error::Error};

use crate::ast_rules::{AstRules, TokenDefinition};

#[derive(Debug)]
pub struct Token {
    pub key: String,
    pub value: String,
    pub line_nr: usize,
    //column: u32,
}

fn consume_tokens(line_nr: usize, stream: &str, tokens: &Vec<(String, Box<dyn TokenDefinition>)>) -> Vec<Token> {
    let mut found_tokens = Vec::new();
    
    for (key, token) in tokens.iter() {
        let (consumed, new_stream) = token.consume(&stream);
        if consumed.len() > 0 {
            found_tokens.push(Token {key: key.clone(), value: String::from(consumed), line_nr});
            if new_stream.len() > 0 {
                found_tokens.append(&mut consume_tokens(line_nr, new_stream.trim(), tokens));
            }
            break;
        }
    }
    found_tokens
}

pub fn lex(file: &str, rules: &AstRules) -> Result<Vec<Token>, Box<dyn Error>> {
    let source = fs::read_to_string(file)?;
    
    let mut found_tokens = Vec::new();
    
    for (line_nr, line) in source.lines().enumerate() {
        found_tokens.append(&mut consume_tokens(line_nr, line.trim(), &rules.tokens));

        /*line.split(' ').filter(|s| s.len() > 0).for_each(|word| {
            found_tokens.append(&mut consume_tokens(line_nr, word, &rules.tokens));
        });*/
    }

    Ok(found_tokens)
}