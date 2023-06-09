#![feature(box_patterns, box_syntax)]

use color_eyre::{eyre::eyre, eyre::ContextCompat, Result};
use im_rc::HashMap;

enum Token {
    App,
    Bool,
    False,
    Fun,
    Lam,
    Nat,
    Pair,
    Prod,
    Succ,
    True,
    U(Level),
    Var(String),
    Zero,
}

type Var<'a> = &'a str;

type Type<'a> = Value<'a>;

type Level = u8;

enum Value<'a> {
    Bool,
    False,
    Fun(Box<Value<'a>>, Box<Value<'a>>),
    Lam(Var<'a>, Box<Value<'a>>),
    Nat,
    Neutral(Neutral<'a>),
    Pair(Box<Value<'a>>, Box<Value<'a>>),
    Prod(Box<Value<'a>>, Box<Value<'a>>),
    Succ(Box<Value<'a>>),
    True,
    U(Level),
    Zero,
}

enum Neutral<'a> {
    App(Box<Neutral<'a>>, Box<Value<'a>>),
    Var(Var<'a>),
}

fn parse_var(input: &[Token]) -> Result<(Var, &[Token])> {
    let (token, input) = input.split_first().wrap_err("unexpected end of input")?;

    let Token::Var(x) = token else {
        return Err(eyre!("invalid var"));
    };

    Ok((x, input))
}

// fn parse_bool(input: &[Token]) -> Result<(Value, &[Token])> {
//     let (token, input) = input.split_first().wrap_err("unexpected end of input")?;
//
//     match token {
//         Token::False => Ok((Value::False, input)),
//         Token::True => Ok((Value::True, input)),
//         _ => Err(eyre!("invalid boolean")),
//     }
// }
//
// fn parse_fun<'a>(ty_a: &Type, ty_b: &Type, input: &'a [Token]) -> Result<(Value<'a>, &'a [Token])> {
//     let (token, input) = input.split_first().wrap_err("unexpected end of input")?;
//
//     match token {
//         Token::Lam => {
//             let (x, input) = parse_var(input)?;
//             let (b, input) = parse_value(ty_b, input)?;
//
//             Ok((Value::Lam(x, box b), input))
//         }
//         _ => Err(eyre!("invalid function")),
//     }
// }
//
// fn parse_nat(input: &[Token]) -> Result<(Value, &[Token])> {
//     let (token, input) = input.split_first().wrap_err("unexpected end of input")?;
//
//     match token {
//         Token::Succ => {
//             let (n, input) = parse_nat(input)?;
//
//             Ok((Value::Succ(box n), input))
//         }
//         Token::Zero => Ok((Value::Zero, input)),
//         _ => Err(eyre!("invalid natural")),
//     }
// }
//
// fn parse_prod<'a>(
//     ty_a: &Type,
//     ty_b: &Type,
//     input: &'a [Token],
// ) -> Result<(Value<'a>, &'a [Token])> {
//     let (token, input) = input.split_first().wrap_err("unexpected end of input")?;
//
//     let Token::Pair = token else {
//         return Err(eyre!("invalid pair"));
//     };
//
//     let (a, input) = parse_value(ty_a, input)?;
//     let (b, input) = parse_value(ty_b, input)?;
//
//     Ok((Value::Pair(box a, box b), input))
// }
//
// fn parse_value<'a>(ty: &Type, input: &'a [Token]) -> Result<(Value<'a>, &'a [Token])> {
//     match ty {
//         Type::Bool => parse_bool(input),
//         Type::Nat => parse_nat(input),
//         Type::Prod(ty_a, ty_b) => parse_prod(ty_a, ty_b, input),
//         _ => unreachable!(),
//     }
// }

fn parse_type(input: &[Token]) -> Result<(Type, &[Token])> {
    let (token, input) = input.split_first().wrap_err("unexpected end of input")?;

    match token {
        Token::Bool => Ok((Type::Bool, input)),
        Token::Fun => {
            let (ty_a, input) = parse_type(input)?;
            let (ty_b, input) = parse_type(input)?;

            Ok((Type::Fun(box ty_a, box ty_b), input))
        }
        Token::Nat => Ok((Type::Nat, input)),
        Token::Prod => {
            let (ty_a, input) = parse_type(input)?;
            let (ty_b, input) = parse_type(input)?;

            Ok((Type::Prod(box ty_a, box ty_b), input))
        }
        _ => Err(eyre!("invalid type")),
    }
}

fn infer(input: &[Token]) -> Result<(Type, &[Token])> {
    let (token, input) = input.split_first().wrap_err("unexpected end of input")?;

    match token {
        Token::App => {
            let (type_f, input) = infer(input)?;

            let Type::Fun(box type_a, box type_b) = type_f else {
                return Err(eyre!("illegal application"));
            };

            let input = check(&type_a, input)?;

            Ok((type_b, input))
        }
        Token::Var(x) => {
            todo!();
        }
        _ => Err(eyre!("failed to infer type")),
    }
}

fn check<'a>(type_: &Type, input: &'a [Token]) -> Result<&'a [Token]> {
    let (token, input) = input.split_first().wrap_err("unexpected end of input")?;

    match (type_, token) {
        (Type::Bool, Token::False) => Ok(input),
        (Type::Bool, Token::True) => Ok(input),
        (Type::Fun(box _type_a, box type_b), Token::Lam) => {
            let (_x, input) = parse_var(input)?;
            check(&type_b, input)
        }
        (Type::Nat, Token::Zero) => Ok(input),
        (Type::Nat, Token::Succ) => check(type_, input),
        (Type::Prod(box type_a, box type_b), Token::Pair) => {
            let input = check(type_a, input)?;
            check(type_b, input)
        }
        (Type::U(_i), Token::Bool) => Ok(input),
        (Type::U(_i), Token::Fun) => {
            let input = check(type_, input)?;
            check(type_, input)
        }
        (Type::U(_i), Token::Nat) => Ok(input),
        (Type::U(_i), Token::Prod) => {
            let input = check(type_, input)?;
            check(type_, input)
        }
        _ => {
            let (type__, input) = infer(input)?;

            todo!("check for alpha equality");

            Ok(input)
        }
    }
}

type Env<'a> = HashMap<Var<'a>, Value<'a>>;

fn eval<'a>(input: &'a [Token], env: &Env) -> Result<(Value<'a>, &'a [Token])> {
    let (token, input) = input.split_first().wrap_err("unexpected end of input")?;

    match token {
        Token::App => {
            let (f, input) = eval(input, env)?;
            let (a, input) = eval(input, env)?;

            todo!();
            // match f {
            //     Value::Lam()
            // }
            // Ok((Value::Neutral(Neutral::App(box f, box a)), input))
        }
        Token::Bool => Ok((Value::Bool, input)),
        Token::False => Ok((Value::False, input)),
        Token::Fun => {
            let (type_a, input) = eval(input, env)?;
            let (type_b, input) = eval(input, env)?;
            Ok((Value::Fun(box type_a, box type_b), input))
        }
        Token::Lam => {
            let (x, input) = parse_var(input)?;
            todo!();
        }
        Token::Nat => Ok((Value::Nat, input)),
        Token::Pair => {
            let (a, input) = eval(input, env)?;
            let (b, input) = eval(input, env)?;
            Ok((Value::Pair(box a, box b), input))
        }
        Token::Prod => {
            let (type_a, input) = eval(input, env)?;
            let (type_b, input) = eval(input, env)?;
            Ok((Value::Prod(box type_a, box type_b), input))
        }
        Token::Succ => {
            let (n, input) = eval(input, env)?;
            Ok((Value::Succ(box n), input))
        }
        Token::True => Ok((Value::True, input)),
        &Token::U(i) => Ok((Value::U(i), input)),
        Token::Var(x) => {
            todo!();
        }
        Token::Zero => Ok((Value::Zero, input)),
    }
}
