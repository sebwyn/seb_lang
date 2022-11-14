/*mod ast_rules;
mod dot_visualizer;
mod ast_lexer;
mod ast_parser;

use ast_parser::parse;
use ast_rules::AstRules;
use crate::{dot_visualizer::print_ast, ast_lexer::lex};
*/

use std::error::Error;
use std::fs;

mod parser_lang;

use parser_lang::parse;

fn main() -> Result<(), Box<dyn Error>> {
    //parse a syntax tree config
    /*
    let ast = AstRules::parse("ast_definition.ast")?;
    print_ast("ast.dot", &ast)?;

    let tokens = lex("main.seb", &ast)?;
    parse(&tokens, &ast);
    */
    //command line app
    /*loop {
        let mut input: String = String::new();
        stdin().read_line(&mut input)?;
        input = String::from(input.trim());
        if input == "quit" {
            break
        }

        //parse the input here, and print the debug result
        parse(&input);

    }*/

    let test = fs::read_to_string("test.ast")?;
    parse(&test)?;

    Ok(())
}
