use super::ast::*;
use nom::types::CompleteByteSlice;
use nom::*;
use std::collections::BTreeSet;

// literals

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

named!(number<CompleteByteSlice, Number>,
    map_res!(
        recognize!(
            tuple!(
                opt!(tag!("-")),
                digit,
                opt!(complete!(
                    tuple!(tag!("."), digit)
                )),
                opt!(complete!(
                    tuple!(
                        tag_no_case!("e"),
                        opt!(tag!("-")),
                        digit
                    )
                ))
            )
        ),
        parse_number
    )
);

fn parse_number(b: CompleteByteSlice) -> Result<Number, String> {
    let s = std::str::from_utf8(b.0).map_err(|e| e.to_string())?;
    if s.contains('.') || s.contains('e') || s.contains('E') {
        let n = s
            .parse::<f64>()
            .map(Number::Float)
            .map_err(|e| e.to_string())?;
        Ok(n)
    } else {
        let n = s
            .parse::<i64>()
            .map(Number::Integer)
            .map_err(|e| e.to_string())?;
        Ok(n)
    }
}

named!(string<CompleteByteSlice, String>,
    map_res!(
        alt!(
            delimited!(char!('"'), recognize!(opt!(is_not!("\""))), char!('"'))
            |
            delimited!(char!('\''), recognize!(opt!(is_not!("'"))), char!('\''))
        ),
        parse_string
    )
);

fn parse_string(b: CompleteByteSlice) -> Result<String, Box<dyn std::error::Error>> {
    let s = String::from_utf8(b.0.to_owned())?;
    Ok(s)
}

named!(boolean<CompleteByteSlice, bool>,
    alt!(
        map!(tag_no_case!("true"), |_| true)
        |
        map!(tag_no_case!("false"), |_| false)
    )
);

named!(ident<CompleteByteSlice, String>,
    map_res!(
        recognize!(
            tuple!(
                one_of!("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"),
                many0!(one_of!("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"))
            )
        ),
        parse_string
    )
);

named!(literal_value<CompleteByteSlice, LiteralValue>,
    alt!(
        map!(boolean, LiteralValue::Boolean)
        |
        map!(number, |n| match n {
            Number::Float(v) => LiteralValue::Float(v),
            Number::Integer(v) => LiteralValue::Integer(v),
        })
        |
        map!(string, LiteralValue::String)
        |
        map!(tag_no_case!("none"), |_| LiteralValue::None)
    )
);

// Call

fn split_args<T>(all_args: Vec<(Option<String>, T)>) -> Option<(Vec<T>, Vec<(String, T)>)> {
    let mut args = Vec::new();
    let mut named_args = Vec::new();
    let mut named_arg_was_met = false;
    for (name, arg) in all_args {
        if !named_arg_was_met {
            match name {
                Some(name) => {
                    named_arg_was_met = true;
                    named_args.push((name, arg));
                }
                None => {
                    args.push(arg);
                }
            }
        } else {
            match name {
                Some(name) => named_args.push((name, arg)),
                None => return None, // non-named argument after named one
            }
        }
    }
    Some((args, named_args))
}

fn parse_call((function, all_args): (String, Vec<(Option<String>, Arg)>)) -> Result<Call, String> {
    let (args, named_args) = split_args(all_args).ok_or_else(|| {
        format!(
            "Bad call of {}: positional argument after named one.",
            function
        )
    })?;
    Ok(Call {
        function,
        args,
        named_args,
    })
}

named!(call<CompleteByteSlice, Call>,
    map_res!(
        do_parse!(
            function: ident >>
            tag!("(") >>
            all_args: separated_list!(tag!(","), call_arg) >>
            tag!(")") >>
            (function, all_args)
        ),
        parse_call
    )
);

named!(arg<CompleteByteSlice, Arg>,
    alt!(
        map!(literal_value, Arg::Literal)
        |
        map!(expression, Arg::Expression)
    )
);

named!(call_arg<CompleteByteSlice, (Option<String>, Arg)>,
    tuple!(
        opt!(terminated!(ident, tag!("="))),
        arg
    )
);

// path expression

named!(partial_path_element<CompleteByteSlice, String>,
    map_res!(
        escaped!(
            many1!(one_of!(&br##"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#%&+-/:;<=>?@^_`~"##[..])),
            '\\',
            one_of!(&br#"(){}[],.'"\|=$"#[..])
        ),
        parse_string
    )
);

named!(match_enum<CompleteByteSlice, Vec<String>>,
    delimited!(char!('{'), separated_nonempty_list!(tag!(","), partial_path_element), char!('}'))
);

named!(match_group_range<CompleteByteSlice, BTreeSet<char>>,
    map!(
        do_parse!(
            from_char: none_of!("]") >>
            char!('-') >>
            to_char: none_of!("]") >>
            (from_char as u8, to_char as u8)
        ),
        |(from_char, to_char)| (from_char..=to_char).map(|c| c as char).collect()
    )
);

named!(match_group_single<CompleteByteSlice, BTreeSet<char>>,
    map!(none_of!("]"), |c| {
        [c].iter().cloned().collect()
    })
);

named!(match_group<CompleteByteSlice, BTreeSet<char>>,
    map!(
        do_parse!(
            char!('[') >>
            starts_with_dash: opt!(char!('-')) >>
            chars: fold_many1!(
                alt!(match_group_range | match_group_single),
                BTreeSet::new(),
                |mut acc: BTreeSet<char>, chars| {
                    acc.extend(chars);
                    acc
                }
            ) >>
            ends_with_dash: opt!(char!('-')) >>
            char!(']') >>
            (starts_with_dash.is_some() || ends_with_dash.is_some(), chars)
        ),
        |(add_dash, mut chars): (bool, BTreeSet<char>)| {
            if add_dash {
                chars.insert('-');
            }
            chars
        }
    )
);

named!(path_element<CompleteByteSlice, PathElement>,
    alt!(
        map!(match_enum, PathElement::Enum)
        |
        map!(match_group, PathElement::OneOf)
        |
        map!(char!('*'), |_| PathElement::Asterisk)
        |
        map!(preceded!(tag!("$"), partial_path_element), PathElement::Variable)
        |
        map!(partial_path_element, PathElement::Partial)
    )
);

named!(path_word<CompleteByteSlice, PathWord>,
    map!(
        many1!(path_element),
        PathWord
    )
);

named!(path_expression<CompleteByteSlice, PathExpression>,
    map!(
        separated_nonempty_list!(tag!("."), path_word),
        PathExpression
    )
);

// template

named!(source<CompleteByteSlice, Source>,
    alt!(
        map!(call, Source::Call)
        |
        map!(path_expression, Source::Path)
    )
);

named!(template_arg<CompleteByteSlice, (Option<String>, LiteralValue)>,
    tuple!(
        opt!(terminated!(ident, tag!("="))),
        literal_value
    )
);

fn parse_template(
    (source, all_args): (Source, Option<Vec<(Option<String>, LiteralValue)>>),
) -> Result<Template, String> {
    let (args, named_args) = split_args(all_args.unwrap_or_default()).ok_or_else(|| {
        format!(
            "Bad call of template {:?}: positional argument after named one.",
            source
        )
    })?;
    Ok(Template {
        source,
        args,
        named_args,
    })
}

named!(template<CompleteByteSlice, Template>,
    map_res!(
        do_parse!(
            tag!("template") >>
            tag!("(") >>
            source: source >>
            all_args: opt!(preceded!(tag!(","), separated_nonempty_list!(tag!(","), template_arg))) >>
            tag!(")") >>
            (source, all_args)
        ),
        parse_template
    )
);

// expression

named!(expression_base<CompleteByteSlice, Expression>,
    alt!(
        map!(template, Expression::Template)
        |
        map!(call, Expression::Call)
        |
        map!(path_expression, Expression::Path)
    )
);

fn parse_expression((base, pipe_calls): (Expression, Vec<Call>)) -> Expression {
    fn wrap(base: Expression, mut call: Call) -> Expression {
        call.args.insert(0, Arg::Expression(base));
        Expression::Call(call)
    }
    let mut wrapped = base;
    for call in pipe_calls {
        wrapped = wrap(wrapped, call);
    }
    wrapped
}

named!(expression<CompleteByteSlice, Expression>,
    map!(
        do_parse!(
            base: expression_base >>
            pipe_calls: many0!(preceded!(tag!("|"), call)) >>
            (base, pipe_calls)
        ),
        parse_expression
    )
);

macro_rules! impl_try_from {
    ($parser:ident, $type:ty) => {
        impl std::convert::TryFrom<&[u8]> for $type {
            type Error = String;

            fn try_from(input: &[u8]) -> Result<Self, Self::Error> {
                let (tail, result) =
                    $parser(CompleteByteSlice(input)).map_err(|e| e.to_string())?;

                if tail.len() == 0 {
                    Ok(result)
                } else {
                    Err(format!(
                        "Unexpected input at position {}.",
                        input.len() - tail.len()
                    ))
                }
            }
        }

        impl std::str::FromStr for $type {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                std::convert::TryFrom::try_from(s.as_bytes())
            }
        }
    };
}

impl_try_from!(path_expression, PathExpression);
impl_try_from!(expression, Expression);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse {
        ($parser:ident, $input:tt) => {{
            let (tail, result) = $parser(CompleteByteSlice($input)).unwrap();
            assert_eq!(tail.len(), 0);
            result
        }};
    }

    #[test]
    fn test_number() {
        assert_eq!(parse!(number, b"-3.14e-4"), Number::Float(-0.000_314));
        assert_eq!(parse!(number, b"0"), Number::Integer(0));
        assert_eq!(parse!(number, b"124"), Number::Integer(124));
    }

    #[test]
    fn test_string() {
        assert_eq!(parse!(string, br#""""#), String::new());
        assert_eq!(parse!(string, br#"''"#), String::new());
        assert_eq!(parse!(string, br#""hello""#), String::from("hello"));
        assert_eq!(parse!(string, br#"'hello'"#), String::from("hello"));
    }

    #[test]
    fn test_call() {
        println!("Parsed {:#?}", parse!(call, b"sin()"));
        println!("Parsed {:#?}", parse!(call, b"log(2,32.4)"));
        println!("Parsed {:#?}", parse!(call, br#"A(1,false,x=2,y="hello")"#));
        println!("Parsed {:#?}", parse!(call, br#"B(x=2,y="hello",z=None)"#));
    }

    #[test]
    fn test_template() {
        println!(
            "Parsed {:#?}",
            parse!(template, b"template(emea.events\\[2019\\].clicks)")
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                b"template(emea.events\\[2019\\].clicks,skip_empty=true)"
            )
        );
        println!(
            "Parsed {:#?}",
            parse!(template, b"template(average(emea.events\\[2019\\].clicks))")
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                b"template(average(emea.events\\[2019\\].clicks,n=7))"
            )
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                b"template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false)"
            )
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                b"template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)"
            )
        );
    }

    #[test]
    fn test_path_expression() {
        assert_eq!(
            "a.b.c".parse::<PathExpression>().unwrap(),
            PathExpression(vec![
                PathWord(vec![PathElement::Partial("a".to_string()),]),
                PathWord(vec![PathElement::Partial("b".to_string()),]),
                PathWord(vec![PathElement::Partial("c".to_string()),]),
            ])
        );
        assert_eq!(
            parse!(path_expression, b"a.b.c").to_string().as_str(),
            "a.b.c"
        );
        assert_eq!(
            parse!(path_expression, b"a.b.[0-9]").to_string().as_str(),
            "a.b.[0123456789]"
        );
        assert_eq!(
            parse!(path_expression, b"a.b.[0-9_A-F-]")
                .to_string()
                .as_str(),
            "a.b.[-0123456789ABCDEF_]"
        );
        assert_eq!(
            parse!(path_expression, b"a.[cat]").to_string().as_str(),
            "a.[act]"
        );
        assert_eq!(
            parse!(path_expression, b"hosts.$hostname.cpu")
                .to_string()
                .as_str(),
            "hosts.$hostname.cpu"
        );
        assert_eq!(
            "hosts.$hostname.cpu".parse::<PathExpression>().unwrap(),
            PathExpression(vec![
                PathWord(vec![PathElement::Partial("hosts".to_string()),]),
                PathWord(vec![PathElement::Variable("hostname".to_string()),]),
                PathWord(vec![PathElement::Partial("cpu".to_string()),]),
            ])
        );

        assert_eq!(
            parse!(path_expression, b"alpha.beta.gamma")
                .to_string()
                .as_str(),
            "alpha.beta.gamma"
        );
        assert_eq!(
            parse!(path_expression, b"alpha.*.gamma")
                .to_string()
                .as_str(),
            "alpha.*.gamma"
        );
        assert_eq!(
            parse!(path_expression, b"alpha.*.gamma"),
            PathExpression(vec![
                PathWord(vec![PathElement::Partial("alpha".to_string()),]),
                PathWord(vec![PathElement::Asterisk,]),
                PathWord(vec![PathElement::Partial("gamma".to_string()),]),
            ])
        );

        assert_eq!(
            parse!(path_expression, b"emea.events.clicks{2018,2019}")
                .to_string()
                .as_str(),
            "emea.events.clicks{2018,2019}"
        );
        assert_eq!(
            parse!(
                path_expression,
                br"emea.events\[\]\{\}.clicks{2018,2019}.05"
            )
            .to_string()
            .as_str(),
            "emea.events\\[\\]\\{\\}.clicks{2018,2019}.05"
        );
    }

    #[test]
    fn test_expression() {
        println!("Parsed {:#?}", parse!(expression, b"template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)|aliasByNode(1)|movingAverage(\"5min\")"));
    }
}
