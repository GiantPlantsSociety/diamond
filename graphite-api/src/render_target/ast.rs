use std::collections::BTreeSet;
use std::fmt;

// Literal

#[derive(Debug)]
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
                    PathElement::Variable(ref s) => write!(f, "${}", s)?,
                    PathElement::Partial(ref s) => write!(f, "{}", s)?,
                    PathElement::Asterisk => write!(f, "*")?,
                    PathElement::OneOf(ref chars) => {
                        write!(f, "[")?;
                        for ch in chars.iter() {
                            write!(f, "{}", ch)?;
                        }
                        write!(f, "]")?;
                    }
                    PathElement::Enum(ref e) => {
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

#[derive(Debug)]
pub struct Call {
    pub function: String,
    pub args: Vec<Arg>,
    pub named_args: Vec<(String, Arg)>,
}

#[derive(Debug)]
pub enum Arg {
    Literal(LiteralValue),
    Expression(Expression),
}

// Template

#[derive(Debug)]
pub enum Source {
    Call(Call),
    Path(PathExpression),
}

#[derive(Debug)]
pub struct Template {
    pub source: Source,
    pub args: Vec<LiteralValue>,
    pub named_args: Vec<(String, LiteralValue)>,
}

// Expression

#[derive(Debug)]
pub enum Expression {
    Path(PathExpression),
    Call(Call),
    Template(Template),
}
