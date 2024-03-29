use super::ast::*;
use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_not, tag, tag_no_case};
use nom::character::complete::{char as c, digit1, none_of, one_of};
use nom::combinator::{map, map_res, opt, recognize};

use nom::error::{convert_error, VerboseError, VerboseErrorKind};
use nom::multi::{fold_many1, many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{Err, IResult};
use std::collections::BTreeSet;

// literals

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

fn literal_number(input: &str) -> IResult<&str, LiteralValue, VerboseError<&str>> {
    let (input, number) = map_res(
        recognize(tuple((
            opt(c('-')),
            digit1,
            opt(tuple((c('.'), digit1))),
            opt(tuple((one_of("eE"), opt(c('-')), digit1))),
        ))),
        parse_number,
    )(input)?;

    match number {
        Number::Float(v) => Ok((input, LiteralValue::Float(v))),
        Number::Integer(v) => Ok((input, LiteralValue::Integer(v))),
    }
}

fn literal_string(input: &str) -> IResult<&str, LiteralValue, VerboseError<&str>> {
    let (input, string) = alt((
        delimited(c('"'), recognize(opt(is_not("\""))), c('"')),
        delimited(c('\''), recognize(opt(is_not("'"))), c('\'')),
    ))(input)?;

    Ok((input, LiteralValue::String(string.to_owned())))
}

fn literal_boolean(input: &str) -> IResult<&str, LiteralValue, VerboseError<&str>> {
    let (input, boolean) = alt((
        map(tag_no_case("true"), |_| true),
        map(tag_no_case("false"), |_| false),
    ))(input)?;

    Ok((input, LiteralValue::Boolean(boolean)))
}

fn literal_none(input: &str) -> IResult<&str, LiteralValue, VerboseError<&str>> {
    let (input, _) = tag_no_case("none")(input)?;
    Ok((input, LiteralValue::None))
}

fn literal_value(input: &str) -> IResult<&str, LiteralValue, VerboseError<&str>> {
    alt((
        literal_boolean,
        literal_number,
        literal_string,
        literal_none,
    ))(input)
}

fn ident(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (input, word) = recognize(tuple((
        one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"),
        many0(one_of(
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_",
        )),
    )))(input)?;

    Ok((input, word.to_owned()))
}

fn call_arg(input: &str) -> IResult<&str, (Option<String>, Arg), VerboseError<&str>> {
    tuple((opt(terminated(ident, c('='))), arg))(input)
}

fn arg(input: &str) -> IResult<&str, Arg, VerboseError<&str>> {
    alt((
        map(literal_value, Arg::Literal),
        map(expression, Arg::Expression),
    ))(input)
}

fn call_args(
    input: &str,
) -> IResult<&str, (String, Vec<(Option<String>, Arg)>), VerboseError<&str>> {
    let (input, function) = ident(input)?;
    let (input, _) = c('(')(input)?;
    let (input, all_args) = separated_list0(c(','), call_arg)(input)?;
    let (input, _) = c(')')(input)?;

    Ok((input, (function, all_args)))
}

fn call(input: &str) -> IResult<&str, Call, VerboseError<&str>> {
    map_res(call_args, parse_call)(input)
}

fn parse_number(s: &str) -> Result<Number, String> {
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

fn parse_call(argv: (String, Vec<(Option<String>, Arg)>)) -> Result<Call, String> {
    let (function, all_args) = argv;

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

// Path Expression
fn partial_path_element(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (inp, out) = escaped_transform(
        one_of(
            r##"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#%&+-/:;<=>?@^_`~"##,
        ),
        '\\',
        one_of(r#"(){}[],.'"\|=$"#),
    )(input)?;
    if !out.is_empty() {
        Ok((inp, out))
    } else {
        Err(Err::Error(VerboseError {
            errors: vec![(
                inp,
                VerboseErrorKind::Context("Path element cannot be empty"),
            )],
        }))
    }
}

fn path_element_enum(input: &str) -> IResult<&str, Vec<String>, VerboseError<&str>> {
    delimited(
        c('{'),
        separated_list1(tag(","), partial_path_element),
        c('}'),
    )(input)
}

fn match_group_range(input: &str) -> IResult<&str, BTreeSet<char>, VerboseError<&str>> {
    let (input, from_char) = none_of("]")(input)?;
    let (input, _) = c('-')(input)?;
    let (input, to_char) = none_of("]")(input)?;

    let range = ((from_char as u8)..=(to_char as u8))
        .map(|c| c as char)
        .collect();

    Ok((input, range))
}

fn match_group_single(input: &str) -> IResult<&str, BTreeSet<char>, VerboseError<&str>> {
    let (input, single) = none_of("]")(input)?;
    let group_single = [single].iter().cloned().collect();
    Ok((input, group_single))
}

fn path_element_group(input: &str) -> IResult<&str, BTreeSet<char>, VerboseError<&str>> {
    let (input, _) = c('[')(input)?;
    let (input, start_dash) = opt(c('-'))(input)?;

    let (input, mut chars) = fold_many1(
        alt((match_group_range, match_group_single)),
        BTreeSet::new,
        |mut acc: BTreeSet<char>, chars: BTreeSet<char>| {
            acc.extend(chars);
            acc
        },
    )(input)?;

    let (input, end_dash) = opt(c('-'))(input)?;
    let (input, _) = c(']')(input)?;

    if start_dash.is_some() || end_dash.is_some() {
        chars.insert('-');
    }

    Ok((input, chars))
}

fn path_element(input: &str) -> IResult<&str, PathElement, VerboseError<&str>> {
    alt((
        map(path_element_enum, PathElement::Enum),
        map(path_element_group, PathElement::OneOf),
        map(c('*'), |_| PathElement::Asterisk),
        map(
            preceded(c('$'), partial_path_element),
            PathElement::Variable,
        ),
        map(partial_path_element, PathElement::Partial),
    ))(input)
}

fn path_word(input: &str) -> IResult<&str, PathWord, VerboseError<&str>> {
    let (input, path_word) = many1(path_element)(input)?;
    Ok((input, PathWord(path_word)))
}

fn path_expression(input: &str) -> IResult<&str, PathExpression, VerboseError<&str>> {
    let (input, path) = separated_list1(c('.'), path_word)(input)?;
    Ok((input, PathExpression(path)))
}

// template
fn source(input: &str) -> IResult<&str, Source, VerboseError<&str>> {
    alt((map(call, Source::Call), map(path_expression, Source::Path)))(input)
}

fn template_arg(input: &str) -> IResult<&str, (Option<String>, LiteralValue), VerboseError<&str>> {
    let (input, arg) = opt(terminated(ident, c('=')))(input)?;
    let (input, value) = literal_value(input)?;
    Ok((input, (arg, value)))
}

fn parse_template(
    source: Source,
    all_args: Option<Vec<(Option<String>, LiteralValue)>>,
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

fn template_internal(
    input: &str,
) -> IResult<&str, (Source, Option<Vec<(Option<String>, LiteralValue)>>), VerboseError<&str>> {
    let (input, _) = tag("template")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, source) = source(input)?;

    let (input, all_args) =
        opt(preceded(tag(","), separated_list1(tag(","), template_arg)))(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, (source, all_args)))
}

fn template(input: &str) -> IResult<&str, Template, VerboseError<&str>> {
    map_res(template_internal, |(source, all_args)| {
        parse_template(source, all_args)
    })(input)
}

// expression
fn parse_expression(base: Expression, pipe_calls: Vec<Call>) -> Expression {
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

fn expression(input: &str) -> IResult<&str, Expression, VerboseError<&str>> {
    let (input, base) = alt((
        map(template, Expression::Template),
        map(call, Expression::Call),
        map(path_expression, Expression::Path),
    ))(input)?;

    let (input, pipe_calls) = many0(preceded(tag("|"), call))(input)?;
    let expression = parse_expression(base, pipe_calls);
    Ok((input, expression))
}

macro_rules! impl_try_from {
    ($parser:ident, $type:ty) => {
        impl std::convert::TryFrom<&str> for $type {
            type Error = String;

            fn try_from(input: &str) -> Result<Self, Self::Error> {
                let (tail, result) = $parser(input).map_err(|e| match e {
                    Err::Error(e) => format!("ERROR {}", convert_error(input, e)),
                    Err::Failure(e) => format!("FAILURE {}", convert_error(input, e)),
                    Err::Incomplete(_) => "Input is incomplete".to_owned(),
                })?;

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
                std::convert::TryFrom::try_from(s)
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
            let (tail, result) = $parser($input).unwrap();
            assert_eq!(tail.len(), 0, "Incomplete parse");
            result
        }};
    }

    impl_try_from!(literal_value, LiteralValue);
    impl_try_from!(call, Call);
    impl_try_from!(path_word, PathWord);

    #[test]
    fn test_number() {
        assert_eq!(parse_number("-3.14e-4").unwrap(), Number::Float(-0.000_314));
        assert_eq!(parse_number("0").unwrap(), Number::Integer(0));
        assert_eq!(parse_number("124").unwrap(), Number::Integer(124));
    }

    #[test]
    fn test_literal() {
        assert_eq!(
            "-3.14e-4".parse::<LiteralValue>().unwrap(),
            LiteralValue::Float(-0.000_314)
        );
        assert_eq!(parse!(literal_value, "0"), LiteralValue::Integer(0));
        assert_eq!(parse!(literal_value, "124"), LiteralValue::Integer(124));
        assert_eq!(
            parse!(literal_value, r#""""#),
            LiteralValue::String(String::new())
        );
        assert_eq!(
            parse!(literal_value, r#"''"#),
            LiteralValue::String(String::new())
        );
        assert_eq!(
            parse!(literal_value, r#""hello""#),
            LiteralValue::String(String::from("hello"))
        );
        assert_eq!(
            parse!(literal_value, r#"'hello'"#),
            LiteralValue::String(String::from("hello"))
        );
    }

    #[test]
    fn test_call() {
        assert_eq!(
            "sin()".parse::<Call>().unwrap(),
            Call {
                function: "sin".to_owned(),
                args: vec![],
                named_args: vec![],
            }
        );
        assert_eq!(
            "log(2,32.4)".parse::<Call>().unwrap(),
            Call {
                function: "log".to_owned(),
                args: vec![
                    Arg::Literal(LiteralValue::Integer(2)),
                    Arg::Literal(LiteralValue::Float(32.4)),
                ],
                named_args: vec![],
            }
        );
        assert_eq!(
            r#"A(1,false,x=2,y="hello")"#.parse::<Call>().unwrap(),
            Call {
                function: "A".to_owned(),
                args: vec![
                    Arg::Literal(LiteralValue::Integer(1)),
                    Arg::Literal(LiteralValue::Boolean(false)),
                ],
                named_args: vec![
                    ("x".to_owned(), Arg::Literal(LiteralValue::Integer(2))),
                    (
                        "y".to_owned(),
                        Arg::Literal(LiteralValue::String(String::from("hello")))
                    ),
                ],
            }
        );
        assert_eq!(
            r#"B(x=2,y="hello",z=None)"#.parse::<Call>().unwrap(),
            Call {
                function: "B".to_owned(),
                args: vec![],
                named_args: vec![
                    ("x".to_owned(), Arg::Literal(LiteralValue::Integer(2))),
                    (
                        "y".to_owned(),
                        Arg::Literal(LiteralValue::String(String::from("hello")))
                    ),
                    ("z".to_owned(), Arg::Literal(LiteralValue::None)),
                ],
            }
        );
    }

    #[test]
    fn test_template() {
        println!(
            "Parsed {:#?}",
            parse!(template, "template(emea.events\\[2019\\].clicks)")
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                "template(emea.events\\[2019\\].clicks,skip_empty=true)"
            )
        );
        println!(
            "Parsed {:#?}",
            parse!(template, "template(average(emea.events\\[2019\\].clicks))")
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                "template(average(emea.events\\[2019\\].clicks,n=7))"
            )
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                "template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false)"
            )
        );
        println!(
            "Parsed {:#?}",
            parse!(
                template,
                "template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)"
            )
        );
    }

    #[test]
    fn test_partial_path_element() {
        assert_eq!(parse!(partial_path_element, "a").to_string().as_str(), "a");
        assert_eq!(
            parse!(partial_path_element, "abc").to_string().as_str(),
            "abc"
        );
        assert_eq!(
            parse!(partial_path_element, "abc\\[de\\]f123")
                .to_string()
                .as_str(),
            "abc[de]f123"
        );

        assert!(partial_path_element("").is_err());
    }

    #[test]
    fn test_path_element_enum() {
        assert_eq!(
            parse!(path_element_enum, "{a,b,c}"),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(
            parse!(path_element_enum, "{a,ab,abc}"),
            vec!["a".to_string(), "ab".to_string(), "abc".to_string()]
        );
    }

    #[test]
    fn test_path_word_element() {
        let (_, element) = path_element("a").unwrap();
        assert_eq!(element, PathElement::Partial("a".to_string()));

        let (_, element) = path_element("abc").unwrap();
        assert_eq!(element, PathElement::Partial("abc".to_string()));

        assert!(path_element("").is_err());
    }

    #[test]
    fn test_path_word() {
        let (_, word) = path_word("a").unwrap();
        assert_eq!(word, PathWord(vec![PathElement::Partial("a".to_string())]));

        let (_, word) = path_word("abc").unwrap();
        assert_eq!(
            word,
            PathWord(vec![PathElement::Partial("abc".to_string())])
        );
    }

    #[test]
    fn test_path_expression() {
        assert!("a..b".parse::<PathExpression>().is_err());
        assert_eq!(
            "a.b.c".parse::<PathExpression>().unwrap(),
            PathExpression(vec![
                PathWord(vec![PathElement::Partial("a".to_string()),]),
                PathWord(vec![PathElement::Partial("b".to_string()),]),
                PathWord(vec![PathElement::Partial("c".to_string()),]),
            ])
        );
        assert_eq!(
            parse!(path_expression, "a.b.c").to_string().as_str(),
            "a.b.c"
        );
        assert_eq!(
            parse!(path_expression, "a.b.[0-9]").to_string().as_str(),
            "a.b.[0123456789]"
        );
        assert_eq!(
            parse!(path_expression, "a.b.[0-9_A-F-]")
                .to_string()
                .as_str(),
            "a.b.[-0123456789ABCDEF_]"
        );
        assert_eq!(
            parse!(path_expression, "a.[cat]").to_string().as_str(),
            "a.[act]"
        );
        assert_eq!(
            parse!(path_expression, "hosts.$hostname.cpu")
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
            parse!(path_expression, "alpha.beta.gamma")
                .to_string()
                .as_str(),
            "alpha.beta.gamma"
        );
        assert_eq!(
            parse!(path_expression, "alpha.*.gamma")
                .to_string()
                .as_str(),
            "alpha.*.gamma"
        );
        assert_eq!(
            parse!(path_expression, "alpha.*.gamma"),
            PathExpression(vec![
                PathWord(vec![PathElement::Partial("alpha".to_string()),]),
                PathWord(vec![PathElement::Asterisk,]),
                PathWord(vec![PathElement::Partial("gamma".to_string()),]),
            ])
        );

        assert_eq!(
            parse!(path_expression, "emea.events.clicks{2018,2019}")
                .to_string()
                .as_str(),
            "emea.events.clicks{2018,2019}"
        );
        assert_eq!(
            parse!(path_expression, r"emea.events\[\]\{\}.clicks{2018,2019}.05")
                .to_string()
                .as_str(),
            "emea.events[]{}.clicks{2018,2019}.05"
        );
    }

    #[test]
    fn test_expression() {
        println!("Parsed {:#?}", parse!(expression, "template(average(emea.events\\[2019\\].clicks,n=7),skip_empty=false,none=none)|aliasByNode(1)|movingAverage(\"5min\")"));
    }
}
