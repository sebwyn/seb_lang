use std::{
    collections::HashMap,
    error::Error,
    fs,
    iter::Peekable,
    path::Path,
};

/*
    A bunk parser for defining real parsers
*/

use regex::Regex;

pub trait Token {
    //fn consume(&self);
    fn characters(&self) -> &str;
}
pub struct ExactToken {
    pub characters: String,
}
impl ExactToken {
    fn new(characters: &str) -> Self {
        Self {
            characters: characters.to_string(),
        }
    }
}
impl Token for ExactToken {
    fn characters(&self) -> &str {
        &self.characters
    }
}

pub struct RegToken {
    pub characters: String,
    pub reg: Regex,
}
impl RegToken {
    fn new(characters: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            characters: characters.to_string(),
            reg: Regex::new(characters)?,
        })
    }
}
impl Token for RegToken {
    fn characters(&self) -> &str {
        &self.characters
    }
}

#[derive(Clone, Default)]
pub struct RuleNode {
    pub children: HashMap<String, RuleNode>,
}

pub struct AstRules {
    pub tokens: HashMap<String, Box<dyn Token>>,
    pub rules: HashMap<String, RuleNode>,
}

pub enum ParseErr {
    InvalidTokenDef(String),
    TokenReassign(String),
    TokenInvalidReg(String),
    InvalidRuleDef(String),
    NoRoot,
    ConsumeErr,
}
impl std::fmt::Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseErr")
    }
}
impl std::fmt::Debug for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTokenDef(arg0) => f.debug_tuple("InvalidTokenDef").field(arg0).finish(),
            Self::TokenReassign(arg0) => f.debug_tuple("TokenReassign").field(arg0).finish(),
            Self::TokenInvalidReg(arg0) => f.debug_tuple("TokenInvalidReg").field(arg0).finish(),
            Self::InvalidRuleDef(arg0) => f.debug_tuple("InvalidRuleDef").field(arg0).finish(),
            Self::NoRoot => write!(f, "NoRoot"),
            Self::ConsumeErr => write!(f, "ConsumeErr"),
        }
    }
}

impl Error for ParseErr {}

fn add_rule(node: &mut RuleNode, rule_vec: &[String]) {
    if let Some(name) = rule_vec.get(0) {
        let next_node = if let Some(child) = node.children.get_mut(name) {
            child
        } else {
            node.children.insert(name.clone(), RuleNode::default());
            node.children.get_mut(name).unwrap()
        };

        return add_rule(next_node, rule_vec.split_at(1).1);
    }
}

impl AstRules {
    pub fn parse(path: &str) -> Result<AstRules, Box<dyn Error>> {
        //open the file and begin parsing
        let file = fs::read_to_string(Path::new(path))?;
        //trim whitespace from the file with a filter
        let mut lines = file
            .split('\n')
            .map(|s| s.trim())
            .filter(|line| line.len() != 0)
            .peekable();

        //consume token defs in a loop
        let mut tokens = HashMap::new();
        loop {
            match Self::consume_token_def(&mut lines) {
                Ok((token, characters)) => {
                    let last_token = tokens.insert(token.clone(), characters);
                    if last_token.is_some() {
                        Err(ParseErr::TokenReassign(token))?;
                    }
                }
                Err(ParseErr::ConsumeErr) => break,
                e => {
                    e?;
                }
            }
        }

        //consume rules in a loop
        let mut root: Option<String> = None;
        let mut rules: HashMap<String, RuleNode> = HashMap::new();
        loop {
            match Self::consume_rule(&mut lines) {
                Ok((name, rule_vec, is_root)) => {
                    //add this rule to our rule set
                    if rule_vec.len() > 0 {
                        //get or insert
                        let node = match rules.get_mut(&name) {
                            Some(node) => node,
                            None => {
                                rules.insert(name.clone(), RuleNode::default());
                                rules.get_mut(&name).unwrap()
                            }
                        };
                        add_rule(node, &rule_vec);

                        if is_root {
                            root = Some(name);
                        }
                    } else {
                        Err(ParseErr::InvalidRuleDef(format!(
                            "name: {}, rule: {:?}",
                            name, rule_vec
                        )))?
                    }
                }
                Err(ParseErr::ConsumeErr) => break,
                e => {
                    e?;
                }
            }
        }

        Ok(Self {
            tokens,
            rules,
        })
    }

    fn consume_token_def<'a, T: Iterator<Item = &'a str>>(
        lines: &mut Peekable<T>,
    ) -> Result<(String, Box<dyn Token>), ParseErr> {
        if let Some(line) = lines.peek() {
            if line.chars().nth(0).unwrap() == '#' {
                //let mut line = lines.next().unwrap();
                let line = line.clone().split_at(1).1;

                let (token, characters) = line
                    .split_once(' ')
                    .ok_or(ParseErr::InvalidTokenDef(line.to_string()))?;
                if characters.len() == 0 {
                    Err(ParseErr::InvalidTokenDef(line.to_string()))?
                }

                return if token
                    .chars()
                    .nth(0)
                    .ok_or(ParseErr::InvalidTokenDef(line.to_string()))?
                    == '!'
                {
                    let token = token.split_at(1).1;
                    let reg_token = RegToken::new(characters)
                        .map_err(|e| ParseErr::TokenInvalidReg(line.to_string()))?;
                    lines.next();
                    Ok((token.to_string(), Box::new(reg_token)))
                } else {
                    lines.next();
                    Ok((token.to_string(), Box::new(ExactToken::new(characters))))
                };
            }
        }
        Err(ParseErr::ConsumeErr)
    }

    //no limits on what this will try to consume
    fn consume_rule<'a, T: Iterator<Item = &'a str>>(
        lines: &mut Peekable<T>,
    ) -> Result<(String, Vec<String>, bool), ParseErr> {
        if let Some(line) = lines.peek() {
            let (name, rule) = line
                .split_once(':')
                .ok_or(ParseErr::InvalidRuleDef(line.to_string()))?;
            //check if the name is prefixed with a *

            let (name, is_root) = if name
                .chars()
                .nth(0)
                .ok_or(ParseErr::InvalidRuleDef(line.to_string()))?
                == '*'
            {
                (name.split_at(1).1, true)
            } else {
                (name, false)
            };

            //trim the rules
            let rule = rule.trim();

            let name = name.to_string();
            let rule_vec: Vec<String> = rule.split(' ').map(|s| s.to_string()).collect();

            lines.next();
            return Ok((name, rule_vec, is_root));
        }

        Err(ParseErr::ConsumeErr)
    }
}
