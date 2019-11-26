use nom::error::{ErrorKind, ParseError};
use nom::{
    AsChar, ExtendInto, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice,
};
use nom::{Err, IResult};
use std::ops::RangeFrom;

pub fn parser_escaped_transform<Input, Error, F, G, O1, O2, ExtendItem, Output>(
    normal: F,
    control_char: char,
    transform: G,
    input: Input,
) -> IResult<Input, Output, Error>
where
    Input: Clone
        + Offset
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>
        + InputIter,
    Input: ExtendInto<Item = ExtendItem, Extender = Output>,
    O1: ExtendInto<Item = ExtendItem, Extender = Output>,
    O2: ExtendInto<Item = ExtendItem, Extender = Output>,
    Output: core::iter::Extend<<Input as ExtendInto>::Item>,
    Output: core::iter::Extend<<O1 as ExtendInto>::Item>,
    Output: core::iter::Extend<<O2 as ExtendInto>::Item>,
    <Input as InputIter>::Item: AsChar,
    F: Fn(Input) -> IResult<Input, O1, Error>,
    G: Fn(Input) -> IResult<Input, O2, Error>,
    Error: ParseError<Input>,
{
    let mut index = 0;
    let mut res = input.new_builder();

    if input.input_len() == 0 {
        return Err(Err::Error(Error::from_error_kind(
            input,
            ErrorKind::EscapedTransform,
        )));
    }

    let i = input.clone();

    while index < i.input_len() {
        let remainder = i.slice(index..);
        match normal(remainder.clone()) {
            Ok((i2, o)) => {
                o.extend_into(&mut res);
                if i2.input_len() == 0 {
                    return Ok((i.slice(i.input_len()..), res));
                } else {
                    index = input.offset(&i2);
                }
            }
            Err(Err::Error(_)) => {
                // unwrap() should be safe here since index < $i.input_len()
                if remainder.iter_elements().next().unwrap().as_char() == control_char {
                    let next = index + control_char.len_utf8();
                    let input_len = input.input_len();

                    if next >= input_len {
                        return Err(Err::Error(Error::from_error_kind(
                            remainder,
                            ErrorKind::EscapedTransform,
                        )));
                    } else {
                        match transform(i.slice(next..)) {
                            Ok((i2, o)) => {
                                o.extend_into(&mut res);
                                if i2.input_len() == 0 {
                                    return Ok((i.slice(i.input_len()..), res));
                                } else {
                                    index = input.offset(&i2);
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    if index == 0 {
                        return Err(Err::Error(Error::from_error_kind(
                            remainder,
                            ErrorKind::EscapedTransform,
                        )));
                    }
                    return Ok((remainder, res));
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok((input.slice(index..), res))
}
