use std::error::Error;
use std::{fs, env};
use std::io::stdin;

use seb_lang::dot_visualizer::print_sym_tree;
use seb_lang::parser_lang::parse;

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = env::args().skip(1).next();

    if let Some(file) = file_name {
        let test = fs::read_to_string(file)?;
        let rules = parse(&test)?;
        print_sym_tree("parser.dot", &rules)?;

    } else {
        //command line app
        loop {
            let mut input: String = String::new();
            stdin().read_line(&mut input)?;
            input = String::from(input.trim());
            if input == "quit" {
                break
            }

            //parse the input here, and print the debug result
            parse(&input)?;
        }
    }

    Ok(())
}
