use enum_derive::ParseEnumError;
use crate::uxn::{InstructionMode, Opcode};
use nom::branch::{alt, permutation};
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace1, none_of, one_of};
use nom::combinator::{map, map_res, not, opt, recognize, value};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{count, many0_count, many1, many_till};
use nom::sequence::{pair, preceded, tuple};
use nom::{error, IResult, Parser};

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
    value((), tuple((tag("("), take_until(")"), tag(")"))))(i)
}

pub fn hexadecimal(input: &str) -> IResult<&str, u16> {
    map_res(
        // recognize returns the consumed value as result, not the actual token result
        recognize(many1(one_of("0123456789abcdefABCDEF"))),
        |out: &str| u16::from_str_radix(out, 16),
    )(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

/// Either Or Parser: Will return `success_value` if `f` is successful, `fail_value` if not.
pub fn either_or<I: Clone, O: Clone, O2, E: ParseError<I>, F>(success_value: O, fail_value: O, mut f: F)
                                                              -> impl FnMut(I) -> IResult<I, O, E>
    where F: Parser<I, O2, E>,
{
    move |input: I| {
        f.parse(input.clone()).map_or(
            Ok((input, fail_value.clone())),
            |(i, _)| Ok((i, success_value.clone())),
        )
    }
}

// actual uxntal elements

pub fn ascii_literal(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("\""), many1(not(multispace1))))(input)
}


pub fn address(input: &str) -> IResult<&str, Address> {
    let (input, (mode, address)) = tuple((
        alt((
            value(AddressingMode::LiteralRelative, tag(",")),
            value(AddressingMode::LiteralZeroPage, tag(".")),
            value(AddressingMode::RawAbsolute, tag(":")),
            value(AddressingMode::LiteralAbsolute, tag(";")),
        )),
        hexadecimal,
    ))(input)?;
    Ok((input, Address { mode, address }))
}

pub fn label(input: &str) -> IResult<&str, Label> {
    let (input, (type_, name)) = tuple((
        alt((
            value(LabelType::Parent, tag("@")),
            value(LabelType::Child, tag(":")),
        )),
        identifier,
    ))(input)?;
    Ok((
        input,
        Label {
            name: name.to_string(),
            type_,
        },
    ))
}

pub fn immediate(input: &str) -> IResult<&str, Instruction> {
    map_res(
        nom::sequence::preceded(tag("#"), hexadecimal),
        |v| -> Result<Instruction, &str> {
            Ok(Instruction {
                opcode: Opcode::LIT,
                mode: if v > 0xFF {
                    InstructionMode::Keep | InstructionMode::Short
                } else {
                    InstructionMode::Keep
                },
                immediate: v,
            })
        },
    )(input)
}

pub fn instruction_mode_flags(input: &str) -> IResult<&str, InstructionMode> {
    map(permutation(
        (
            either_or(InstructionMode::Short, InstructionMode::None, char('2')),
            either_or(InstructionMode::Keep, InstructionMode::None, char('k')),
            either_or(InstructionMode::Return, InstructionMode::None, char('r')),
        )
    ),
        |(v1, v2, v3): (InstructionMode, InstructionMode, InstructionMode)| v1 | v2 | v3,
    )(input)
}

pub fn instruction(input: &str) -> IResult<&str, Instruction> {
    let opcode_without_lit = map_res(recognize(count(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), 3)), |v: &str| -> Result<Opcode, &str> {
        if v.eq("LIT") {
            return Err("LIT needs to be parsed at a higher level");
        }
        v.parse().or(Err("Could not parse opcode"))
    });
    let standard_instructions = map(pair(
        opcode_without_lit,
        instruction_mode_flags,
    ), |(opcode, mode)| Instruction { opcode, mode, immediate: 0x00 });

    let lit = map(pair(
        preceded(tag("LIT"), instruction_mode_flags),
        preceded(multispace1, hexadecimal)),
                  |(mode, immediate)| Instruction {
                      opcode: Opcode::LIT,
                      mode: mode | InstructionMode::Keep,
                      immediate,
                  });
    alt((
        standard_instructions,
        lit))(input)
}

#[test]
fn parse_either_or() {
    let result: IResult<&str, u32> = either_or(1, 0, char('1'))("1");
    assert_eq!(
        result,
        Ok(("", 1))
    );

    let result: IResult<&str, u32> = either_or(1, 0, char('1'))("2");
    assert_eq!(
        result,
        Ok(("2", 0))
    );

    let result: IResult<&str, (u32, u32, u32)> = permutation(
        (
            either_or(1, 0, char('1')),
            either_or(2, 0, char('2')),
            either_or(3, 0, char('3')),
        ))("234");
    assert_eq!(
        result,
        Ok(("4", (0, 2, 3)))
    );
}

#[test]
fn parse_immediate() {
    assert_eq!(
        immediate("#18"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::LIT,
                mode: InstructionMode::Keep,
                immediate: 0x18,
            }
        ))
    );
    assert_eq!(
        immediate("#1818"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::LIT,
                mode: InstructionMode::Keep | InstructionMode::Short,
                immediate: 0x1818,
            }
        ))
    );
}

#[test]
fn parse_instruction() {
    assert_eq!(
        instruction("DUP"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::DUP,
                mode: InstructionMode::None,
                immediate: 0x00,
            }
        ))
    );

    assert_eq!(
        instruction("DUP2"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::DUP,
                mode: InstructionMode::Short,
                immediate: 0x00,
            }
        ))
    );

    assert_eq!(
        instruction("DUP2r"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::DUP,
                mode: InstructionMode::Short | InstructionMode::Return,
                immediate: 0x00,
            }
        ))
    );

    assert_eq!(
        instruction("LIT 12"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::LIT,
                mode: InstructionMode::Keep,
                immediate: 0x12,
            }
        ))
    );

    assert_eq!(
        instruction("LIT2 1234"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::LIT,
                mode: InstructionMode::Short | InstructionMode::Keep,
                immediate: 0x1234,
            }
        ))
    );

    assert_eq!(
        instruction("LIT2r 1234"),
        Ok((
            "",
            Instruction {
                opcode: Opcode::LIT,
                mode: InstructionMode::Short | InstructionMode::Keep | InstructionMode::Return,
                immediate: 0x1234,
            }
        ))
    );
}
