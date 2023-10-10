mod expression;

use std::{path::{Path, PathBuf}, rc::Rc, fmt::{Display, Debug}, str::FromStr, iter::Peekable};

use self::expression::{Expression, collect_expr, Op, Value};

pub fn assemble(file_in: impl AsRef<Path>, file_out: impl AsRef<Path>) -> Result<(), Error> {
    let instrs = parse_file(file_in)?;
    for (_, i) in instrs {
        println!("{i:?}")
    }
    Ok(())
}

fn parse_file<'a>(file: impl AsRef<Path>) -> Result<Vec<(Loc, Instruction)>, Error> {
    let lines = read_lines(file)?;
    let tokens = tokenize_lines(lines)?;
    let mut instrs = vec![];
    for (loc, line) in tokens {
        let instr = instructionize(line, Some(&loc))?;
        instrs.push((loc, instr));
    }
    Ok(instrs)
}

fn instructionize(tokens: Vec<Token>, loc: Option<&Loc>) -> Result<Instruction, Error> {
    macro_rules! assert_ended {
        ($index: expr) => { {
            if tokens.len() < $index { panic!("Index should be less than or equal to tokens length! Try testing for a smaller index!") }
            if tokens.len() == $index { Ok(()) }
            else { Err(Error(format!("Invalid instruction syntax, surplus argument {:?}", tokens[$index]), loc.cloned())) }
        }? };
    }
    macro_rules! get {
        ($index: expr) => {
            tokens.get($index).ok_or_else(|| Error(format!("Invalid instruction syntax, expected another argument"), loc.cloned()))?
        };
        (? $index: expr) => {
            tokens.get($index)
        };
        ($index: expr => $variant: ident) => {
            match tokens.get($index).ok_or_else(|| Error(format!("Invalid instruction syntax, expected another argument"), loc.cloned()))? {
                Token::$variant(v) => Ok(v),
                other => Err(Error(format!("Expected token type {}, got {:?}", stringify!($variant), other), loc.cloned()))
            }?
        };
        (? $index: expr => $variant: ident) => {
            match tokens.get($index) {
                Some(Token::$variant(v)) => Some(v),
                _ => None
            }
        };
    }
    Ok(match get!(0) {
        Token::Control('$') => { 
            let label = get!(1 => Ascii); 
            let (expr, i) = collect_expr(&tokens, 1, loc)?;
            assert_ended!(i);
            Instruction::Variable(label.to_string(), expr, None)
        },
        Token::Control('@') => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Location(expr) },
        Token::Control('.') => if let Some(dtype) = get!(? 1 => Ident) {
            match dtype.as_str() {
                "ascii" => {
                    let ascii = get!(2 => Ascii);
                    assert_ended!(3);
                    Instruction::Data(Data::Ascii(ascii.clone()))
                }
                "f32" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::F32(Box::new(expr), None)) },
                "u32" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::U32(Box::new(expr), None)) },
                "i32" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::I32(Box::new(expr), None)) },
                "u16" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::U16(Box::new(expr), None)) },
                "i16" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::I16(Box::new(expr), None)) },
                "u8" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::U8(Box::new(expr), None)) },
                "i8" => { let (expr, i) = collect_expr(&tokens, 2, loc)?; assert_ended!(i); Instruction::Data(Data::I8(Box::new(expr), None)) },
                _ => Err(Error(format!("Invalid data type `.{dtype}`"), loc.cloned()))?
            }
        } else { Err(Error(format!("Invalid instruction syntax, expected `.ascii`, `.f32`, `.u32`, `.i32`, `.u16`, `.i16`, `.u8` or `.i8`"), loc.cloned()))? },
        Token::Ident(ident) => if get!(? 1 => Control) == Some(&':') {
            assert_ended!(2);
            Instruction::Label(ident.clone())
        } else { 
            Instruction::Command()
        },
        _ => Err(Error(format!("Invalid instruction syntax"), loc.cloned()))?
    })
}

fn tokenize_lines(lines: Vec<(Loc, String)>) -> Result<Vec<(Loc, Vec<Token>)>, Error> {
    let mut tokens = vec![];
    for (loc, line) in lines {
        let t = tokenize(&line, Some(&loc))?;
        if t.len() == 0 { continue; }
        tokens.push((loc, t));
    }
    Ok(tokens)
}

fn read_lines(file: impl AsRef<Path>) -> Result<Vec<(Loc, String)>, Error> {
    let file = Rc::new(file.as_ref().to_owned());
    let raw = Rc::new(std::fs::read_to_string(&*file).map_err(|_| Error(format!("Unable to read file: `{}`", file.to_str().unwrap()), None))?);
    let mut lines = vec![];
    for (i, line) in raw.split('\n').enumerate() {
        lines.push((Loc { code: raw.clone(), file: file.clone(), line: i}, line.split("//").next().unwrap().trim().to_string()));
    }
    Ok(lines)
}

#[derive(Debug, Clone)]
pub(crate) struct Loc {
    code: Rc<String>,
    file: Rc<PathBuf>,
    line: usize
}

#[derive(Debug)]
enum Instruction {
    Variable(String, Expression, Option<Value>),
    Location(Expression),
    Label(String),
    Command(),
    Data(Data)
}

#[derive(Debug)]
enum Data {
    Ascii(String),
    F32(Box<Expression>, Option<f32>),
    U32(Box<Expression>, Option<u32>),
    I32(Box<Expression>, Option<i32>),
    U16(Box<Expression>, Option<u16>),
    I16(Box<Expression>, Option<i16>),
    U8(Box<Expression>, Option<u8>),
    I8(Box<Expression>, Option<i8>)
}

impl Data {
    fn ty(&self) -> &'static str {
        match self {
            Data::Ascii(_) => "ascii",
            Data::F32(_, _) => "f32",
            Data::U32(_, _) => "u32",
            Data::I32(_, _) => "i32",
            Data::U16(_, _) => "u16",
            Data::I16(_, _) => "i16",
            Data::U8(_, _) => "u8",
            Data::I8(_, _) => "i8",
        }
    }
}

#[derive(Debug)]
enum Arg {
    Register(u32),
    Literal(u32)
}



#[derive(Debug, PartialEq)]
pub(crate)enum Token {
    Ident(String),
    UnsignedInteger(u32, u32),
    SignedInteger(i32, u32),
    Float(f32),
    Ascii(String),
    Control(char)
}

fn tokenize(s: &str, loc: Option<&Loc>) -> Result<Vec<Token>, Error> {
    let mut tokens = vec![];
    let mut iter = s.chars().peekable();
    while let Some(&c) = iter.peek() {
        match c {
            c if !c.is_ascii() => Err(Error(format!("Invalid char `{c}`. Only ascii allowed."), loc.cloned()))?,
            c if c.is_whitespace() => { iter.next(); },
            c if c.is_numeric() => tokens.push(str_to_num_lit(&collect_word(&mut iter), loc)?),
            c if c.is_alphabetic() || c == '_' => tokens.push(Token::Ident(collect_word(&mut iter))),
            '"' => {
                iter.next();
                let mut string = String::new();
                let mut escaped = false;
                while let Some(&c) = iter.peek() {
                    if c == '"' && !escaped { break; }
                    if c == '\\' { escaped = !escaped; }
                    else { escaped = false; }
                    string.push(c);
                    iter.next();
                }
                if iter.next().is_none() { Err(Error(format!("Unexpected end of line while in string literal"), loc.cloned()))? }
                tokens.push(unescape_str(&string, loc)?)
            },
            c => { tokens.push(Token::Control(c)); iter.next(); } ,
        }
    }
    Ok(tokens)
}

pub fn collect_word(iter: &mut Peekable<impl Iterator<Item = char>>) -> String {
    let mut out = String::new();
    while let Some(&c) = iter.peek() {
        if !(c.is_alphanumeric() || c == '_') { break; }
        out.push(c);
        iter.next();
    }
    return out;
}

fn unescape_str(str: &str, loc: Option<&Loc>) -> Result<Token, Error>{
    let mut out = String::new();
    let mut chars = str.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(c) = chars.next() {
                match c {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\'' => out.push('\''),
                    '0' => out.push('\0'),
                    '\\' => out.push('\\'),
                    _ => Err(Error(format!("Invalid escape sequence `\\{c}`"), loc.cloned()))?
                }
            }
        } else {
            out.push(c);
        }
    }
    Ok(Token::Ascii(out))
}

fn str_to_num_lit(n: &str, loc: Option<&Loc>) -> Result<Token, Error>{
    let mut num = n.replace('_', "");
    if num.len() == 0 { Err(Error(format!("Invalid number literal `{n}`"), loc.cloned()))?; }
    let signed = if num.chars().last().unwrap() == 'i' {
        num.pop();
        true
    } else { false };
    if num.len() == 0 { Err(Error(format!("Invalid number literal `{n}`"), loc.cloned()))?; }
    let radix = if num.len() > 2 {
        if num.chars().nth(0).unwrap() == '0' {
            let r = match num.chars().nth(1).unwrap() {
                'b' => Some(0b10), // binary
                'q' => Some(4),    // quaternal
                'o' => Some(0o10), // octal
                'z' => Some(12),   // dozenal
                'x' => Some(0x10), // hexadecimal
                'd' => Some(10),   // decimal
                c if c.is_numeric() => None, // decimal (but no removal needed)
                c => Err(Error(format!("Invalid radix `{c}` for number literal `{n}`"), loc.cloned()))?,
            };
            if let Some(r) = r {
                num.remove(0);
                num.remove(0);
                r
            } else { 10 }
        } else { 10 }
    } else { 10 };
    let float_like = num.contains('.');
    if float_like && (radix != 10 || signed) {
        Err(Error(format!("Expected radix 10 for float literal `{n}`, found {radix}"), loc.cloned()))?;
    }
    let lit = if float_like {
        f32::from_str(&num).map(|f| Token::Float(f)).map_err(|_|
            Error(format!("Invalid float literal `{n}`"), loc.cloned())
        )
    } else {
        if !signed {
            u32::from_str_radix(&num, radix).map(|i|Token::UnsignedInteger(i, radix)).map_err(|_|
                Error(format!("Invalid unsigned integer literal `{n}` with radix {radix}"), loc.cloned())
            )
        } else {
            i32::from_str_radix(&num, radix).map(|i|Token::SignedInteger(i, radix)).map_err(|_|
                Error(format!("Invalid signed integer literal `{n}` with radix {radix}"), loc.cloned())
            )
        }
        
    }?;
    Ok(lit)
}

pub struct Error(String, Option<Loc>);

impl Error {
    fn at(mut self, loc: Loc) -> Self {
        self.1 = Some(loc);
        self
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assembling Error:{}{}", match &self.1 {
            Some(loc) => {
                let start = loc.line.saturating_sub(2);
                let n = loc.line - start + 3;
                let lines = loc.code.split('\n').enumerate()
                    .skip(start).take(n)
                    .map(|(i, l)| if i == loc.line { format!(">{:3} | {l}", i + 1) } else { format!("{:4} | {l}", i + 1) })
                    .collect::<Vec<_>>().join("\n");
                format!("\n@{}:{}\n{}", loc.file.to_str().unwrap(), loc.line + 1, lines)
            }
            None => String::new()
        }, format!("\n  {}", self.0))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n{self}\n")
    }
}