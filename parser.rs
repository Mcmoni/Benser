use super::{Color, Declaration, Rule, Selector, SimpleSelector, Stylesheet, Unit, Value};

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    /// Parse a whole CSS stylesheet.
    pub fn parse(source: &str) -> Stylesheet {
        let mut parser = Parser {
            pos: 0,
            input: source.to_owned(),
        };
        Stylesheet {
            rules: parser.parse_rules(),
        }
    }

    /// Parse a list of rule sets, separated by optional whitespace.
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    /// Parse a rule set: `<selectors> { <declarations> }`.
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    /// Parse a comma-separated list of selectors.
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        // Return selectors with highest specificity first, for use in matching.
        selectors.sort_by_key(|b| std::cmp::Reverse(b.specificity()));
        selectors
    }

    /// Parse one simple selector, e.g.: `type#id.class1.class2.class3`
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    /// Parse a list of declarations enclosed in `{ ... }`.
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    /// Parse one `<property>: <value>;` declaration.
    fn parse_declaration(&mut self) -> Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        Declaration {
            name: property_name,
            value,
        }
    }

    // Methods for parsing values:

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| matches!(c, '0'..='9' | '.'));
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::ColorValue(Color::new(
            self.parse_hex_pair(),
            self.parse_hex_pair(),
            self.parse_hex_pair(),
            255,
        ))
    }

    /// Parse two hexadecimal digits.
    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    /// Parse a property name or keyword.
    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Consume characters until `test` returns false.
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    /// Return the current character, and advance self.pos to the next character.
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn valid_identifier_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_line() {
        assert_eq!(
            Parser::parse("h1, h2, h3 { margin: auto; color: #cc0000; }"),
            Stylesheet {
                rules: vec![Rule {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag_name: Some("h1".to_string()),
                            id: None,
                            class: Vec::new()
                        }),
                        Selector::Simple(SimpleSelector {
                            tag_name: Some("h2".to_string()),
                            id: None,
                            class: Vec::new()
                        }),
                        Selector::Simple(SimpleSelector {
                            tag_name: Some("h3".to_string()),
                            id: None,
                            class: Vec::new()
                        })
                    ],
                    declarations: vec![
                        Declaration {
                            name: "margin".to_string(),
                            value: Value::Keyword("auto".to_string())
                        },
                        Declaration {
                            name: "color".to_string(),
                            value: Value::ColorValue(Color::new(204, 0, 0, 255))
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn two_lines() {
        assert_eq!(
            Parser::parse(
                "div.note { margin-bottom: 20px; padding: 10px; }
                 #answer { display: none; }"
            ),
            Stylesheet {
                rules: vec![
                    Rule {
                        selectors: vec![Selector::Simple(SimpleSelector {
                            tag_name: Some("div".to_string()),
                            id: None,
                            class: vec!["note".to_string()]
                        }),],
                        declarations: vec![
                            Declaration {
                                name: "margin-bottom".to_string(),
                                value: Value::Length(20.0, Unit::Px)
                            },
                            Declaration {
                                name: "padding".to_string(),
                                value: Value::Length(10.0, Unit::Px)
                            }
                        ]
                    },
                    Rule {
                        selectors: vec![Selector::Simple(SimpleSelector {
                            tag_name: None,
                            id: Some("answer".to_string()),
                            class: vec![]
                        }),],
                        declarations: vec![Declaration {
                            name: "display".to_string(),
                            value: Value::Keyword("none".to_string())
                        },]
                    }
                ]
            }
        );
    }
}
