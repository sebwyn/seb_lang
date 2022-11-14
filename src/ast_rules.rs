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

pub trait TokenDefinition {
    //fn consume(&self);
    fn characters(&self) -> &str;
    fn consume<'a>(&self, stream: &'a str) -> (&'a str, &'a str);
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

impl TokenDefinition for ExactToken {
    fn characters(&self) -> &str {
        &self.characters
    }
    fn consume<'a>(&self, stream: &'a str) -> (&'a str, &'a str) {
        if self.characters.len() <= stream.len() {
            let (potential_word, next_stream) = stream.split_at(self.characters.len());
            if potential_word == self.characters {
                return (potential_word, next_stream)
            }
        }
        ("", stream)
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
impl TokenDefinition for RegToken {
    fn characters(&self) -> &str {
        &self.characters
    }
    fn consume<'a>(&self, stream: &'a str) -> (&'a str, &'a str) {
        let found = self.reg.find(stream);
        if let Some(m) = found {
            if m.start() == 0 {
                return stream.split_at(m.end());
            }
        }
        ("", stream)
    }
}

#[derive(Clone, Default, Debug)]
pub struct RuleNode {
    pub priority: u32,
    pub terminal: bool,
    pub children: HashMap<String, RuleNode>,
}
impl RuleNode {
    fn new(priority: u32) -> Self {
        Self {
            priority,
            ..Default::default()
        }
    }
}

pub struct AstRules {
    pub tokens: Vec<(String, Box<dyn TokenDefinition>)>,
    pub rules: HashMap<String, RuleNode>,
    pub root_rule: String,
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

fn add_rule(priority: u32, node: &mut RuleNode, rule_vec: &[String]) -> bool {
    if let Some(name) = rule_vec.get(0) {
        let next_node = if let Some(child) = node.children.get_mut(name) {
            child
        } else {
            node.children.insert(name.clone(), RuleNode::new(priority));
            node.children.get_mut(name).unwrap()
        };
        next_node.terminal = add_rule(priority, next_node, rule_vec.split_at(1).1);
        false
    } else {
        true
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
        let mut tokens = Vec::new();
        loop {
            match Self::consume_token_def(&mut lines) {
                Ok((token, definition)) => {
                    let last_token = tokens.push((token.clone(), definition));
                    /*if last_token.is_some() {
                        Err(ParseErr::TokenReassign(token))?;
                    }*/
                }
                Err(ParseErr::ConsumeErr) => break,
                e => {
                    e?;
                }
            }
        }

        //consume rules in a loop
        let mut root: Option<String> = None;
        let mut rule_count: HashMap<String, u32> = HashMap::new();
        let mut rules: HashMap<String, RuleNode> = HashMap::new();
        loop {
            match Self::consume_rule(&mut lines) {
                Ok((name, rule_vec, is_root)) => {
                    //add this rule to our rule set
                    if rule_vec.len() > 0 {
                        //priority means nothing at the root level
                        let node = match rules.get_mut(&name) {
                            Some(node) => node,
                            None => {
                                rules.insert(name.clone(), RuleNode::new(0));
                                rules.get_mut(&name).unwrap()
                            }
                        };
                        let priority = match rule_count.get_mut(&name) {
                            Some(priority) => priority,
                            None => {
                                rule_count.insert(name.clone(), 0);
                                rule_count.get_mut(&name).unwrap()
                            }
                        };
                        //add one for this new rule
                        *priority += 1;

                        add_rule(*priority, node, &rule_vec);

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
            root_rule: root.expect("No root found in the ast definition file! Use * to define a root")
        })
    }

    fn consume_token_def<'a, T: Iterator<Item = &'a str>>(
        lines: &mut Peekable<T>,
    ) -> Result<(String, Box<dyn TokenDefinition>), ParseErr> {
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
