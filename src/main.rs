use std::str::FromStr;
use pa_rs::parser::*;
use std::io::{self, BufRead};

const REGISTERS: [u16; 16] = [0u16; 16];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15
}

impl FromStr for Register {
    type Err = String;

    fn from_str(s: &str) -> Result<Register, String> {
        use Register::*;
        Ok(match s {
            "R0" => R0,
            "R1" => R1,
            "R2" => R2,
            "R3" => R3,
            "R4" => R4,
            "R5" => R5,
            "R6" => R6,
            "R7" => R7,
            "R8" => R8,
            "R9" => R9,
            "R10" => R10,
            "R11" => R11,
            "R12" => R12,
            "R13" => R13,
            "R14" => R14,
            "R15" => R15,
            x => Err(format!("Invalid register name: {x}! Unreachable error though!"))?
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct U16(u16);

#[derive(Debug, PartialEq, Eq, Clone)]
enum Indexing {
    Num(U16, Register),
    Label(Label, Register),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Sigma16Instruction {
    op: Sigma16Operation,
    instruction: Sigma16InstructionType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Sigma16InstructionType {
    RRR(RRRInstruction),
    RX(RXInstruction),
    CMP(ComparisonInstruction),
    JMP(JumpInstruction),
    //Exp(ExpInstruction),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct RRRInstruction(Register, Register, Register);

#[derive(Debug, PartialEq, Eq, Clone)]
struct RXInstruction(Register, Indexing);

#[derive(Debug, PartialEq, Eq, Clone)]
struct DataInstruction(U16);

#[derive(Debug, PartialEq, Eq, Clone)]
struct ComparisonInstruction(Register, Register);

#[derive(Debug, PartialEq, Eq, Clone)]
struct JumpInstruction(Indexing);

#[derive(Debug, PartialEq, Eq, Clone)]
enum Sigma16Operation {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    AddC,
    MulN,
    DivN,
    // Memory Access
    Lea,
    Load,
    Store,
    Stacks,
    Push,
    Pop,
    Top,
    // Stack Frames
    Save,
    Restore,
    // Jumps
    Jump,
    JumpC0,
    JumpC1,
    Jal,
    JumpZ,
    JumpNZ,
    JumpGE,
    // Logic
    Inv,
    And2,
    Or2,
    Xor2,
    Nand2,
    Nor2,
    // Unknown (for now)
    Trap,
}

impl FromStr for Sigma16Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Sigma16Operation::*;
        Ok(match s {
            "add" => Add,
            "sub" => Sub,
            "div" => Div,
            "lea" => Lea,
            "load" => Load,
            "store" => Store,
            "stacks" => Stacks,
            "push" => Push,
            "pop" => Pop,
            "top" => Top,
            "save" => Save,
            "restore" => Restore,
            "jump" => Jump,
            "jumpc0" => JumpC0,
            "jumpc1" => JumpC1,
            "jal" => Jal,
            "jumpz" => JumpZ,
            "jumpnz" => JumpNZ,
            "jumpge" => JumpGE,
            "inv" => Inv,
            "and2" => And2,
            "or2" => Or2,
            "xor2" => Xor2,
            "nand2" => Nand2,
            "nor2" => Nor2,
            "cmp" => Cmp,
            "trap" => Trap,
            x => Err(format!("Invalid command: {x}! I am unreachable!"))?,
        })
    }
}

#[derive(Clone, Copy)]
struct Sigma16InstructionParser;
impl Parse for Sigma16InstructionParser {
    type Result = Sigma16Instruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        Sigma16OperationParser.sbws()
            .and(Sigma16InstructionTypeParser.sbws())
            .map(|(op, instruction)| Self::Result {
                op,
                instruction,
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct Sigma16OperationParser;
impl Parse for Sigma16OperationParser {
    type Result = Sigma16Operation;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        use Sigma16Operation::*;

        one_of_p([
            str_p("add"),
            str_p("sub"),
            str_p("mul"),
            str_p("div"),
            str_p("cmp"),
            str_p("addc"),
            str_p("muln"),
            str_p("divn"),
            str_p("lea"),
            str_p("load"),
            str_p("store"),
            str_p("stacks"),
            str_p("push"),
            str_p("pop"),
            str_p("top"),
            str_p("save"),
            str_p("restore"),
            str_p("jumpc0"),
            str_p("jumpc1"),
            str_p("jal"),
            str_p("jumpz"),
            str_p("jumpnz"),
            str_p("jumpge"),
            str_p("jump"),
            str_p("inv"),
            str_p("and2"),
            str_p("or2"),
            str_p("xor2"),
            str_p("nand2"),
            str_p("nor2"),
            str_p("trap"),
        ])
            .map(|x| Sigma16Operation::from_str(&x).unwrap())
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct Sigma16InstructionTypeParser;
impl Parse for Sigma16InstructionTypeParser {
    type Result = Sigma16InstructionType;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RXInstructionParser
            .or(RRRInstructionParser)
            .or(ComparisonInstructionParser)
            .or(JumpInstructionParser)
            .map(|either| match either {
                Either::Right(jmp) => Sigma16InstructionType::JMP(jmp),
                Either::Left(either) => match either {
                    Either::Right(cmp) => Sigma16InstructionType::CMP(cmp),
                    Either::Left(either) => match either {
                        Either::Right(rrr) => Sigma16InstructionType::RRR(rrr),
                        Either::Left(rx) => Sigma16InstructionType::RX(rx),
                    }
                }
            })
            //.map(|x| Sigma16InstructionType::RX(x))
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct RRRInstructionParser;
impl Parse for RRRInstructionParser {
    type Result = RRRInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RegisterParser
            .and(char_p(',').drop(RegisterParser))
            .and(char_p(',').drop(RegisterParser))
            .map(|((r1, r2), r3)| RRRInstruction(r1, r2, r3))
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct IndexingParser;
impl Parse for IndexingParser {
    type Result = Indexing;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        U16Parser
            .or(LabelParser)
            .and(char_p('[').drop(RegisterParser))
            .and(char_p(']'))
            .map(|((either, register), _)| match either {
                Either::Left(u64) => Indexing::Num(u64, register),
                Either::Right(label) => Indexing::Label(label, register),
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct RXInstructionParser;
impl Parse for RXInstructionParser {
    type Result = RXInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RegisterParser
            .and(char_p(',').drop(IndexingParser))
            .map(|(Rd, indexing)| RXInstruction(Rd, indexing))
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct DataInstructionParser;
impl Parse for DataInstructionParser {
    type Result = DataInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        str_p("data")
            .drop(whitespace().drop(U16Parser))
            .map(|u64| DataInstruction(u64))
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct ComparisonInstructionParser;
impl Parse for ComparisonInstructionParser {
    type Result = ComparisonInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RegisterParser
            .and(char_p(',').drop(RegisterParser))
            .map(|(rl, rr)| ComparisonInstruction(rl, rr))
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct JumpInstructionParser;
impl Parse for JumpInstructionParser {
    type Result = JumpInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        IndexingParser
            .map(|idx| JumpInstruction(idx))
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct U16Parser;
impl Parse for U16Parser {
    type Result = U16;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        uint_p()
            .or(char_p('$').drop(FourDigitHexParser))
            .or(char_p('-').drop(uint_p()))
            .map(|either| match either {
                Either::Right(neg) => U16(std::u16::MAX - neg as u16),
                Either::Left(either) => match either {
                    Either::Right(num) => U16(num),
                    Either::Left(num) => U16(num as u16),
                }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct FourDigitHexParser;
impl Parse for FourDigitHexParser {
    type Result = u16;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        HexParser
            .and(HexParser)
            .and(HexParser)
            .and(HexParser)
            .map(|(((a, b), c), d)| a << 12 | b << 8 | c << 4 | d)
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct HexParser;
impl Parse for HexParser {
    type Result = u16;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        one_of_p([
            char_p('0'),
            char_p('1'),
            char_p('2'),
            char_p('3'),
            char_p('4'),
            char_p('5'),
            char_p('6'),
            char_p('7'),
            char_p('8'),
            char_p('9'),
            char_p('a'),
            char_p('b'),
            char_p('c'),
            char_p('d'),
            char_p('e'),
            char_p('f'),
        ])
        .map(|x| match x {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'a' => 10,
            'b' => 11,
            'c' => 12,
            'd' => 13,
            'e' => 14,
            'f' => 15,
            x => unreachable!()
        })
        .parse(input)
    }
}

#[derive(Clone, Copy)]
struct RegisterParser;
impl Parse for RegisterParser {
    type Result = Register;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        one_of_p([
            str_p("R10"),
            str_p("R11"),
            str_p("R12"),
            str_p("R13"),
            str_p("R14"),
            str_p("R15"),
            str_p("R0"),
            str_p("R1"),
            str_p("R2"),
            str_p("R3"),
            str_p("R4"),
            str_p("R5"),
            str_p("R6"),
            str_p("R7"),
            str_p("R8"),
            str_p("R9"),
        ])
            .map(|x| Register::from_str(&x).unwrap())
            .parse(input)
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
struct Comment(String);

#[derive(Clone, Copy)]
struct CommentParser;
impl Parse for CommentParser {
    type Result = Comment;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p(';')
            .drop(any_char_p().parse_while(|x| *x != '\n').keep(char_p('\n')))
            .map(|chars| Comment(chars.into_iter().collect()))
            .parse(input)
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
struct Label(String);

#[derive(Clone, Copy)]
struct LabelParser;
impl Parse for LabelParser {
    type Result = Label;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        one_of_p(('a'..='z').chain('A'..='Z').map(|x| char_p(x)))
            .one_or_more()
            .map(|chars| Label(chars.into_iter().collect()))
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct CodeAndCommentParser<CodeParser: Parse + Copy>(CodeParser);
impl<CodeParser: Parse + Copy> Parse for CodeAndCommentParser<CodeParser> {
    type Result = (CodeParser::Result, Comment);

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        self
            .0
            .clone()
            .sbws()
            .and(CommentParser.sbws())
            .parse(input)
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
enum Object {
    Comment(Comment),
    Label(Label),
    LabelAndComment(Label, Comment),
    Instruction(Sigma16Instruction),
    InstructionAndComment(Sigma16Instruction, Comment),
    DataInstruction(DataInstruction),
    Empty,
}

struct ObjectParser;
impl Parse for ObjectParser {
    type Result = Object;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        CommentParser.sbws()
            //.or(CodeAndCommentParser(Sigma16InstructionParser).sbws())
            .or(DataInstructionParser.sbws())
            .or(Sigma16InstructionParser.sbws())
            //.or(CodeAndCommentParser(LabelParser).sbws())
            .or(LabelParser.sbws())
            //.or(whitespace())
            .map(|either| {
                //match either {
                //    Either::Right(x) => {println!("{x:?}"); Object::Empty},
                //    Either::Left(either) =>
                    match either {
                        Either::Right(label) => Object::Label(label),
                        Either::Left(either) => match either {
                            //Either::Right((label, comment)) => Object::LabelAndComment(label, comment),
                            //Either::Left(either) => match either {
                                Either::Right(instruction) => Object::Instruction(instruction),
                                Either::Left(either) => match either {
                                    Either::Right(data_instruction) => Object::DataInstruction(data_instruction),
                                    //Either::Left(either) => match either {
                                     //   Either::Right((instruction, comment)) => Object::InstructionAndComment(instruction, comment),
                                        Either::Left(comment) => Object::Comment(comment),
                                    },
                       //         }
                            }
                        //}
                    //}
                }
            })
            .parse(input)
    }
}

fn main() {
    let mut program = std::fs::read_to_string("examples/code1.txt").unwrap();
    let ast = ObjectParser.zero_or_more().run(&program).unwrap();
    println!("{ast:#?}");
}
