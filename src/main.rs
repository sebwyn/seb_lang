mod ast_rules;
mod dot_visualizer;
mod ast_lexer;

use std::error::Error;

use ast_rules::AstRules;

use crate::{dot_visualizer::print_ast, ast_lexer::lex};

fn main() -> Result<(), Box<dyn Error>> {
    //parse a syntax tree config
    let rules = AstRules::parse("ast_definition.ast")?;
    print_ast("ast.dot", &rules)?;

    lex("main.seb", &rules);

    println!("Hello, world!");

    Ok(())
}
