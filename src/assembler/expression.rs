use std::collections::HashMap;

use super::{Token, Error, Loc, Data};

pub(crate) fn collect_expr(tokens: &Vec<Token>, start: usize, loc: Option<&Loc>) -> Result<(Expression, usize), Error> {
    macro_rules! get {
        ($index: expr) => {
            tokens.get($index).ok_or_else(|| Error(format!("Invalid expression syntax, expected another argument"), loc.cloned()))?
        };
        ($index: expr => $variant: ident) => {
            match tokens.get($index).ok_or_else(|| Error(format!("Invalid expression syntax, expected another argument"), loc.cloned()))? {
                Token::$variant(v) => Ok(v),
                other => Err(Error(format!("Expected token type `{}` for expression, got {:?}", stringify!($variant), other), loc.cloned()))
            }?
        };
    }
    Ok(match get!(start) {
        Token::Ident(var) => {
            if let Some(Token::Control('(')) = tokens.get(start + 1) {
                let mut args  = vec![];
                let mut index = start + 2;
                loop {
                    let (expr, i) = collect_expr(tokens, index, loc)?;
                    index = i;
                    args.push(expr);
                    match get!(index) {
                        Token::Control(')') => break,
                        Token::Control(',') => (),
                        other => Err(Error(format!("Expected `,` or `)` got `{:?}`", other), loc.cloned()))?
                    }
                    index += 1;
                }
                index += 1;
                (Expression::FnCall(var.to_string(), args), index + 1)
            } else {
                (Expression::Variable(var.to_string()), start + 1)
            }
        }
        Token::UnsignedInteger(u, _) => (Expression::Value(Value::UnsignedInteger(*u)), start + 1),
        Token::SignedInteger(i, _) => (Expression::Value(Value::SignedInteger(*i)), start + 1),
        Token::Float(f) => (Expression::Value(Value::Float(*f)), start + 1),
        Token::Control('~') => { let (e, s) = collect_expr(tokens, start + 1, loc)?; (Expression::UnaryOp(UnaryOp::Not, Box::new(e)), s)},
        Token::Control('-') => { let (e, s) = collect_expr(tokens, start + 1, loc)?; (Expression::UnaryOp(UnaryOp::Neg, Box::new(e)), s)},
        Token::Control('(') => {
            #[derive(Debug)]
            enum ExprElem {
                Expr(Expression),
                Op(Op)
            }
            let mut elems = vec![]; 
            let mut index = start + 1;
            while get!(index) != &Token::Control(')') {
                if elems.len() % 2 == 0 {
                    let (expr, i) = collect_expr(tokens, index, loc)?;
                    elems.push(ExprElem::Expr(expr));
                    index = i;
                } else {
                    match get!(index) {
                        Token::Control(c) => elems.push(ExprElem::Op(Op::from_char(*c, loc)?)),
                        other => Err(Error(format!("Expected op, got `{:?}`", other), loc.cloned()))?
                    }
                    index += 1;
                }
            }
            index += 1;
            // last elem was an op: there should be an expression following!
            if elems.len() % 2 == 0 {
                Err(Error(format!("Expression ended on `{:?}`, missing right hand operand!", elems.last().unwrap()), loc.cloned()))?
            }
            macro_rules! combine {
                ($elems: ident: $($op: ident),*) => { {
                    let mut new_elems = vec![]; 
                    let mut iter = $elems.into_iter();
                    while let Some(elem) = iter.next() {
                        match elem {
                            $(
                                ExprElem::Op(Op::$op) => {
                                    let a = if let Some(ExprElem::Expr(expr)) = new_elems.pop() { expr } else { unreachable!() };
                                    let b = if let Some(ExprElem::Expr(expr)) = iter.next() { expr } else { unreachable!() };
                                    new_elems.push(ExprElem::Expr(Expression::BinOp(Op::$op, Box::new(a), Box::new(b))))
                                },
                            )*
                            other => new_elems.push(other),
                        }
                    }
                    new_elems
                } }
            }
            let elems = combine!(elems: And, Or, Xor);
            let elems = combine!(elems: Mul, Div, Mod);
            let mut elems = combine!(elems: Add, Sub);
            assert!(elems.len() == 1);
            if let Some(ExprElem::Expr(expr)) = elems.pop() { (expr, index) } else { unreachable!() }
        },
        Token::Control('!') => Err(Error(format!("Invalid token `!` in expression, did you mean: `~` (unary not)?"), loc.cloned()))?,
        t => Err(Error(format!("Invalid token as expression `{:?}`", t), loc.cloned()))?
    })
}

#[derive(Debug)]
pub(crate) enum Expression {
    Variable(String),
    Value(Value),
    UnaryOp(UnaryOp, Box<Expression>),
    BinOp(Op, Box<Expression>, Box<Expression>),
    FnCall(String, Vec<Expression>)
}

impl Expression {
    pub(crate) fn eval(&self, vars: HashMap<String, Value>) -> Value {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum Value {
    UnsignedInteger(u32),
    SignedInteger(i32),
    Float(f32),
}

impl Value {
    fn ty(&self) -> &'static str {
        match self {
            Value::UnsignedInteger(_) => "unsigned integer",
            Value::SignedInteger(_) => "signed integer",
            Value::Float(_) => "float",
        }
    }
}

impl Value {
    fn copy_into(&self, mut dummy: Data, loc: Option<&Loc>) -> Result<Data, Error> {
        let d = dummy.ty();
        match dummy {
            Data::Ascii(_) => Err(Error(format!("Value may not be of type `ascii`, expected `{}`", dummy.ty()), loc.cloned()))?,
            _ => (),
        }
        match self {
            Value::UnsignedInteger(u) => match dummy {
                Data::U32(_, ref mut v) => *v = Some(*u),
                Data::U16(_, ref mut v) => *v = Some((*u).try_into().map_err(|_| Error(format!("`{}` with value `{}`/`{:X}` does not fit into `{}`", self.ty(), u, u, d), loc.cloned()))?),
                Data::U8(_, ref mut v) => *v = Some((*u).try_into().map_err(|_| Error(format!("`{}` with value `{}`/`{:X}` does not fit into `{}`", self.ty(), u, u, d), loc.cloned()))?),
                _ => Err(Error(format!("Expected `{}`, found value `{}`/`{:X}` of kind `{}`", dummy.ty(), u, u, self.ty()), loc.cloned()))?
            },
            Value::SignedInteger(i) => match dummy {
                Data::I32(_, ref mut v) => *v = Some(*i),
                Data::I16(_, ref mut v) => *v = Some((*i).try_into().map_err(|_| Error(format!("`{}` with value `{}`/`{:X}` does not fit into `{}`", self.ty(), i, i, d), loc.cloned()))?),
                Data::I8(_, ref mut v) => *v = Some((*i).try_into().map_err(|_| Error(format!("`{}` with value `{}`/`{:X}` does not fit into `{}`", self.ty(), i, i, d), loc.cloned()))?),
                _ => Err(Error(format!("Expected `{}`, found value `{}`/`{:X}` of kind `{}`", dummy.ty(), i, i, self.ty()), loc.cloned()))?
            },
            Value::Float(f) => match dummy {
                Data::F32(_, ref mut v) => *v = Some(*f),
                _ => Err(Error(format!("Expected `{}`, found value `{}` of kind `{}`", dummy.ty(), f, self.ty()), loc.cloned()))?
            },
        }
        Ok(dummy)
    }
}

#[derive(Debug)]
pub(crate) enum Op {
    Add,
    Sub,

    Mul,
    Div,
    Mod,

    And,
    Or,
    Xor,
}

impl Op {
    fn from_char(c: char, loc: Option<&Loc>) -> Result<Self, Error>{
        Ok(match c {
            '+' => Op::Add,
            '*' => Op::Mul,
            '-' => Op::Sub,
            '/' => Op::Div,
            '&' => Op::And,
            '|' => Op::Or,
            '^' => Op::Xor,
            '%' => Op::Mod,
            _ => Err(Error(format!("Invalid binary operator `{c}` in expression"), loc.cloned()))?
        })
    }
}

#[derive(Debug)]
pub(crate) enum UnaryOp {
    Neg,
    Not
}

impl UnaryOp {
    fn from_char(c: char, loc: Option<&Loc>) -> Result<Self, Error>{
        Ok(match c {
            '-' => UnaryOp::Neg,
            '~' => UnaryOp::Not,
            _ => Err(Error(format!("Invalid unary operator `{c}` in expression"), loc.cloned()))?
        })
    }
}