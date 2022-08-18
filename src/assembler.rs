use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{alpha1, alphanumeric1, multispace1, none_of, one_of};
use nom::combinator::{map_res, not, recognize, value};
use nom::error::ParseError;
use nom::IResult;
use nom::multi::{many0_count, many1, many_till};
use nom::sequence::{pair, tuple};
use crate::uxn::{Opcode, InstructionMode};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Instruction {
    opcode: Opcode,
    mode: InstructionMode,
    immediate: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LabelType {
    Parent,
    Child,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Label {
    name: String,
    type_: LabelType,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AddressingMode {
    LiteralRelative,
    LiteralZeroPage,
    RawAbsolute,
    LiteralAbsolute,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Address {
    mode: AddressingMode,
    address: u16,
}

pub fn inline_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value(
        (),
        tuple((
            tag("("),
            take_until(")"),
            tag(")")
        )),
    )(i)
}

pub fn hexadecimal(input: &str) -> IResult<&str, u16> {
    map_res(
        recognize(
            many1(
                one_of("0123456789abcdefABCDEF")
            )
        ),
        |out: &str| u16::from_str_radix(out, 16),
    )(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )
    )(input)
}

pub fn address(input: &str) -> IResult<&str, Address> {
    let (input, (mode, address)) = tuple((
        alt(
            (
                value(AddressingMode::LiteralRelative, tag(",")),
                value(AddressingMode::LiteralZeroPage, tag(".")),
                value(AddressingMode::RawAbsolute, tag(":")),
                value(AddressingMode::LiteralAbsolute, tag(";"))
            ),
        ),
        hexadecimal
    ))(input)?;
    Ok((input, Address { mode, address }))
}

pub fn label(input: &str) -> IResult<&str, Label> {
    let (input, (type_, name)) = tuple((
        alt((
            value(LabelType::Parent, tag("@")),
            value(LabelType::Child, tag(":")),
        )),
        identifier
    ))(input)?;
    Ok((input, Label { name: name.to_string(), type_ }))
}

pub fn ascii(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            tag("\""),
            many1(not(multispace1))
        )
    )(input)
}

#[test]
fn parse_immediate() {}
