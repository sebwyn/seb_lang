use crate::ast_lexer::Token;
use crate::ast_rules::{AstRules, RuleNode};

fn parse_rule<'a>(
    name: &str,
    tokens: &'a [Token],
    rule: &RuleNode,
    ast: &AstRules,
) -> (bool, &'a [Token]) {
    //sort the children by the priority of the matches
    let mut children = rule
        .children
        .clone()
        .into_iter()
        .collect::<Vec<(String, RuleNode)>>();
    children.sort_by(|(_, node1), (_, node2)| node1.priority.cmp(&node2.priority));

    println!("Parsing in {}", name);

    //iterate over these children and return immediately as soon as we match a child (this works because of priority)
    for (name, rule) in children.iter() {
        println!("Looking for: {}", name);
        //probably some code repetition in here, but basically match a sequence of tokens returning true if the sequence matches
        //otherwise continue looking for rules in this rule node that could match
        if let Some((key, _)) = ast.tokens.iter().find(|t| t.0 == *name) {
            if let Some(token) = tokens.get(0) {
                if token.key == *key {
                    println!("Found token: {}", token.key);
                    let (matched, remaining) = parse_rule(name, tokens.split_at(1).1, rule, ast);
                    if matched {
                        return (true, remaining);
                    }
                }
            }
        } else {
            //this node is probably a name for another rule, and not a token
            let (matched_name, remaining_tokens) = parse_rule(
                name,
                tokens,
                ast.rules
                    .get(name)
                    .expect(&format!("Name not defined in AST: {}", name)),
                ast,
            );

            if matched_name {
                let (matched, remaining) = parse_rule(name, remaining_tokens, rule, ast);
                if matched {
                    return (true, remaining)
                }
            }
        }
    }

    println!("Reached terminal: {}, remaining: {:?}", name, tokens.iter().map(|t| t.value.as_str()).collect::<Vec<&str>>());

    //getting here means none of our rules have been matched
    if rule.terminal {
        (true, tokens)
    } else {
        (false, tokens)
    }
}

pub fn parse(tokens: &[Token], ast: &AstRules) {
    println!("Parsing tokens: {:?}", tokens.iter().map(|t| t.value.as_str()).collect::<Vec<&str>>());

    let (matched, remaining_tokens) = parse_rule(
        &ast.root_rule,
        tokens,
        ast.rules.get(&ast.root_rule).unwrap(),
        ast,
    );
    //assert!()

    let parsing_status = if matched {
        "succeeded"
    } else {
        "failed"
    };

    println!("Parsing {}", parsing_status);
}
