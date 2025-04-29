use std::collections::BTreeSet;
use std::fmt;

// Literal

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    None,
}

// Path expression

#[derive(Debug, PartialEq)]
pub enum PathElement {
    Variable(String),
    Partial(String),
    Asterisk,
    OneOf(BTreeSet<char>),
    Enum(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub struct PathWord(pub Vec<PathElement>);

impl PathWord {
    pub fn matches(&self, text: &str) -> bool {
        match self.to_regex() {
            Ok(re) => re.is_match(text),
            _ => false,
        }
    }

    pub fn to_regex_pattern(&self) -> Result<String, String> {
        fn element_to_regex(e: &PathElement) -> Result<String, String> {
            match e {
                PathElement::Variable(v) => Err(format!(
                    "Variables are forbidden but variable {} is in expression.",
                    v
                )),
                PathElement::Partial(p) => Ok(regex::escape(p)),
                PathElement::Asterisk => Ok(".*?".to_string()),
                PathElement::OneOf(chars) => Ok(chars
                    .iter()
                    .map(|c| regex::escape(&String::from(*c)))
                    .collect::<Vec<_>>()
                    .join("|")),
                PathElement::Enum(arms) => Ok(arms
                    .iter()
                    .map(|a| regex::escape(a))
                    .collect::<Vec<_>>()
                    .join("|")),
            }
        }

        let mut buf = String::from("^");
        for e in &self.0 {
            let part = element_to_regex(e)?;
            buf.push('(');
            buf.push_str(&part);
            buf.push(')');
        }
        buf.push('$');

        Ok(buf)
    }

    pub fn to_regex(&self) -> Result<regex::Regex, String> {
        let pattern = self.to_regex_pattern()?;
        regex::Regex::new(&pattern).map_err(|err| err.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct PathExpression(pub Vec<PathWord>);

impl fmt::Display for PathExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, word) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, ".")?;
            }
            for element in word.0.iter() {
                match element {
                    PathElement::Variable(s) => write!(f, "${}", s)?,
                    PathElement::Partial(s) => write!(f, "{}", s)?,
                    PathElement::Asterisk => write!(f, "*")?,
                    PathElement::OneOf(chars) => {
                        write!(f, "[")?;
                        for ch in chars.iter() {
                            write!(f, "{}", ch)?;
                        }
                        write!(f, "]")?;
                    }
                    PathElement::Enum(e) => {
                        write!(f, "{{")?;
                        for (index, part) in e.iter().enumerate() {
                            if index > 0 {
                                write!(f, ",")?;
                            }
                            write!(f, "{}", part)?;
                        }
                        write!(f, "}}")?;
                    }
                }
            }
        }
        Ok(())
    }
}

// Call

#[derive(Debug, PartialEq)]
pub struct Call {
    pub function: String,
    pub args: Vec<Arg>,
    pub named_args: Vec<(String, Arg)>,
}

#[derive(Debug, PartialEq)]
pub enum Arg {
    Literal(LiteralValue),
    Expression(Expression),
}

// Template

#[derive(Debug, PartialEq)]
pub enum Source {
    Call(Call),
    Path(PathExpression),
}

#[derive(Debug, PartialEq)]
pub struct Template {
    pub source: Source,
    pub args: Vec<LiteralValue>,
    pub named_args: Vec<(String, LiteralValue)>,
}

// Expression

#[derive(Debug, PartialEq)]
pub enum Expression {
    Path(PathExpression),
    Call(Call),
    Template(Template),
}
