use std::str::FromStr;
use pa_rs::parser::*;
use std::io;

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

#[derive(Debug, PartialEq, Eq)]
struct Disp(u64);

#[derive(Debug, PartialEq, Eq)]
struct Sigma16Instruction {
    op: Sigma16Operation,
    instruction: Sigma16InstructionType,
}

#[derive(Debug, PartialEq, Eq)]
enum Sigma16InstructionType {
    RRR(RRRInstruction),
    RX(RXInstruction),
    //Exp(ExpInstruction),
}

#[derive(Debug, PartialEq, Eq)]
struct RRRInstruction(Register, Register, Register);

#[derive(Debug, PartialEq, Eq)]
struct RXInstruction(Register, Disp, Register);

#[derive(Debug, PartialEq, Eq)]
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
    // Logic
    Inv,
    And2,
    Or2,
    Xor2,
    Nand2,
    Nor2
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
            "inv" => Inv,
            "and2" => And2,
            "or2" => Or2,
            "xor2" => Xor2,
            "nand2" => Nand2,
            "nor2" => Nor2,
            x => Err(format!("Invalid command: {x}! I am unreachable!"))?,
        })
    }
}

struct Sigma16InstructionParser;
impl Parse for Sigma16InstructionParser {
    type Result = Sigma16Instruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        Sigma16OperationParser
            .and(char_p(' '))
            .and(Sigma16InstructionTypeParser)
            .map(|((op, _), instruction)| Self::Result {
                op,
                instruction,
            })
            .parse(input)
    }
}

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
            str_p("jump"),
            str_p("jumpc0"),
            str_p("jumpc1"),
            str_p("jal"),
            str_p("jumpz"),
            str_p("jumpnz"),
            str_p("inv"),
            str_p("and2"),
            str_p("or2"),
            str_p("xor2"),
            str_p("nand2"),
            str_p("nor2"),
        ])
            .map(|x| Sigma16Operation::from_str(&x).unwrap())
            .parse(input)
    }
}

struct Sigma16InstructionTypeParser;
impl Parse for Sigma16InstructionTypeParser {
    type Result = Sigma16InstructionType;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RXInstructionParser
            .or(RRRInstructionParser)
            .map(|result| match result {
                Either::Left(rx) => Sigma16InstructionType::RX(rx),
                Either::Right(rrr) => Sigma16InstructionType::RRR(rrr),
            })
            //.map(|x| Sigma16InstructionType::RX(x))
            .parse(input)
    }
}

struct RRRInstructionParser;
impl Parse for RRRInstructionParser {
    type Result = RRRInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RegisterParser
            .sep_by(',')
            .map(|registers| RRRInstruction(registers[0], registers[1], registers[2]))
            .parse(input)
    }
}


struct RXInstructionParser;
impl Parse for RXInstructionParser {
    type Result = RXInstruction;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        RegisterParser
            .and(char_p(',').drop(DispParser))
            .and(char_p('[').drop(RegisterParser))
            .and(char_p(']'))
            .map(|(((Rd, disp), Ra), _)| RXInstruction(Rd, disp, Ra))
            .parse(input)
    }
}

struct DispParser;
impl Parse for DispParser {
    type Result = Disp;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        uint_p()
            .or(char_p('$').drop(FourDigitHexParser))
            .map(|x| match x {
                Either::Left(disp) => Disp(disp),
                Either::Right(disp) => Disp(disp),
            })
            .parse(input)
    }
}

struct FourDigitHexParser;
impl Parse for FourDigitHexParser {
    type Result = u64;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        HexParser
            .and(HexParser)
            .and(HexParser)
            .and(HexParser)
            .map(|(((a, b), c), d)| (a as u64) << 48 | (b as u64) << 32 | (c as u64) << 16 | (d as u64))
            .parse(input)
    }
}

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

fn main() {
    let mut program = vec![];
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        let instruction = line.trim();
        if line.len() == 0 {
            break
        }
        let instruction = Sigma16InstructionParser.run(instruction).unwrap();
        program.push(instruction);
    }
    println!("{program:?}");
}
