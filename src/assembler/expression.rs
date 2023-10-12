use std::collections::HashMap;

use super::{Token, Error, Loc};

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
                (Expression::FnCall(var.to_string(), args), index)
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

macro_rules! matching_ty_op {
    ($ain: expr, $bin: expr, $loc: ident, $( [$($ty: ident)|+]|$a: ident, $b: ident| $expr: expr ),+) => { {
        let ax = $ain;
        let bx = $bin;
        Ok( 
            match (ax, bx) {
                $(
                    $( (Value::$ty($a), Value::$ty($b)) => Value::$ty($expr), )+
                )+
                (a, b) => Err(Error(format!("{} does not equal type {} in expression", a.ty(), b.ty()), $loc.cloned()))?
            }
        )?
    } };
}

pub(crate)fn expr_funcs_map() -> HashMap<&'static str, fn(Vec<Value>, Option<&Loc>) -> Result<Value, Error>> {
    let mut map: HashMap<&'static str, fn(Vec<Value>, Option<&Loc>) -> Result<Value, Error>> = HashMap::new();
    macro_rules! assert_len {
        ($v: ident >= $len: expr, $loc: ident) => {
            if $v.len() < $len { Err(Error(format!("Expected at least {} args, got {}", $v.len(), $len), None))? }
        };
        ($v: ident == $len: expr, $loc: ident) => {
            if $v.len() != $len { Err(Error(format!("Expected exactly {} args, got {}", $v.len(), $len), None))? }
        };
    }
    map.insert("min", |v, loc| { 
        assert_len!(v >= 1, loc); 
        let init = Ok(v[0].clone());
        v.into_iter().fold(init, 
        |acc, a| Ok(matching_ty_op!(acc?, a, loc, [UnsignedInteger | SignedInteger | Float] |a, b| a.min(b))))
    });
    map.insert("max", |v, loc| { 
        assert_len!(v >= 1, loc); 
        let init = Ok(v[0].clone());
        v.into_iter().fold(init, 
        |acc, a| Ok(matching_ty_op!(acc?, a, loc, [UnsignedInteger |SignedInteger | Float] |a, b| a.max(b))))
    });
    map.insert("div_ceil", |v, loc| { 
        assert_len!(v == 2, loc); 
        Ok(matching_ty_op!(&v[0], &v[1], loc, [UnsignedInteger | SignedInteger] |a, b| a.div_ceil(*b)))
    });
    map.insert("align", |v, loc| { 
        assert_len!(v == 2, loc); 
        Ok(matching_ty_op!(&v[0], &v[1], loc, [UnsignedInteger] |a, b| a.div_ceil(*b) * b))
    });
    map
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
    pub(crate) fn eval(&self, vars: &HashMap<String, Value>, funcs: &HashMap<&'static str, fn(Vec<Value>, Option<&Loc>) -> Result<Value, Error>>, loc: Option<&Loc>) -> Result<Value, Error> {
        Ok(match self {
            Expression::Variable(v) => vars.get(v).cloned().ok_or_else(|| Error(format!("Unrecognized variable `{v}`"), loc.cloned()))?,
            Expression::Value(v) => v.clone(), 
            Expression::UnaryOp(op, e) => match e.eval(vars, funcs, loc)? { 
                Value::UnsignedInteger(u) => match op { UnaryOp::Neg =>Err(Error(format!("Cannot use unary neg (-) on type unsigned int `{u}`"), loc.cloned()))?, UnaryOp::Not => Value::UnsignedInteger(!u) }
                Value::SignedInteger(i) =>  match op { UnaryOp::Neg => Value::SignedInteger(-i), UnaryOp::Not => Value::SignedInteger(!i) }
                Value::Float(f) =>  match op { UnaryOp::Neg => Value::Float(-f), UnaryOp::Not => Err(Error(format!("Cannot use unary not (~) on type float `{f}`"), loc.cloned()))? } 
            }
            Expression::BinOp(op, a, b) => 
                matching_ty_op!(a.eval(vars, funcs, loc)?, b.eval(vars, funcs, loc)?, loc, 
                    [UnsignedInteger | SignedInteger] |a, b| match op {
                        Op::Add => a + b,
                        Op::Sub => a - b,
                        Op::Mul => a * b,
                        Op::Div => a / b,
                        Op::Mod => a % b,
                        Op::And => a & b,
                        Op::Or => a | b,
                        Op::Xor => a ^ b,
                    },
                    [Float] |a, b| match op {
                        Op::Add => a + b,
                        Op::Sub => a - b,
                        Op::Mul => a * b,
                        Op::Div => a / b,
                        Op::Mod => a % b,
                        op => Err(Error(format!("Cannot perform op on floats `{a} {op:?} {b}`"), loc.cloned()))?
                    }
                ),
            Expression::FnCall(ident, args) => (funcs.get(ident.as_str()).ok_or_else(|| Error(format!("Unrecognized function `{ident}`"), loc.cloned()))?)(
                args.into_iter().map(|e| e.eval(vars, funcs, loc)).collect::<Result<Vec<_>, _>>()?, loc
            ).map_err(|e| e.at(loc.cloned()))?,
        })
    }
}

#[derive(Debug, Clone)]
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

    pub(crate) fn to_le_bytes(&self) -> [u8;4]{
        match self {
            Value::UnsignedInteger(u) => u.to_le_bytes(),
            Value::SignedInteger(i) => i.to_le_bytes(),
            Value::Float(f) => f.to_le_bytes(),
        }
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