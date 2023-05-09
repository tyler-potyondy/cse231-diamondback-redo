use super::types;
use im::HashSet;
use sexp::*;
use sexp::Atom::*;

use types::Expr;
use types::Op1;
use types::Op2;
use types::Program;
use types::Definition;


pub fn parse_expr(s: &Sexp) -> types::Expr {
    match s {
        Sexp::Atom(I(n)) => { 
            if *n < types::LEAST_VAL || *n > types::GREATEST_VAL {
                panic!("Invalid - Number too large")
            } else {
            Expr::Number(*n as u64)}
        },
        Sexp::Atom(S(var)) => {
            if var == "true" {
                Expr::Boolean(true)
            } else if var == "false" {
                Expr::Boolean(false)
            } else {            
                Expr::Id(String::from(var))
            }
        },
        Sexp::List(vec) => {
            match &vec[..] {
                // add1 operator //
                [Sexp::Atom(S(op)), e] if op == "add1"   => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e))),

                // sub1 operator //
                [Sexp::Atom(S(op)), e] if op == "sub1"   => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e))),

                // isnum operator //
                [Sexp::Atom(S(op)), e] if op == "isnum"  => Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e))),

                // isbool operator //
                [Sexp::Atom(S(op)), e] if op == "isbool" => Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e))),

                // print statement //
                [Sexp::Atom(S(op)), e] if op == "print"  => Expr::UnOp(Op1::Print, Box::new(parse_expr(e))),

                // addition operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "+" => 
                    Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),

                // subtraction operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => 
                    Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),

                // multiplication operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => 
                    Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),

                // Equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "=" => 
                    Expr::BinOp(Op2::Equal, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),

                // Greater than or equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == ">=" => 
                    Expr::BinOp(Op2::GreaterEqual, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),

                // Less than or equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "<=" => {
                    Expr::BinOp(Op2::LessEqual, Box::new(parse_expr(e1)),Box::new(parse_expr(e2)))},

                // Greater than operator //
                [Sexp::Atom(S(op)), e1, e2] if op == ">" => 
                    Expr::BinOp(Op2::Greater, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),

                // Less than operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "<" => 
                    Expr::BinOp(Op2::Less, Box::new(parse_expr(e1)),Box::new(parse_expr(e2))),
                
                // if statement //
                [Sexp::Atom(S(op)), e1, e2, e3] if op == "if" => 
                    Expr::If(Box::new(parse_expr(e1)),Box::new(parse_expr(e2)), Box::new(parse_expr(e3))),

                // loop statment //
                [Sexp::Atom(S(op)), e] if op == "loop" =>  
                    Expr::Loop(Box::new(parse_expr(e))),

                // Let statement // 
                [Sexp::Atom(S(op)), Sexp::List(list_vec), e] if op == "let" => {
                    let mut bind_vec = Vec::new();
                    for item in list_vec{
                        bind_vec.push(parse_bind(item))
                    }
                    if bind_vec.len() == 0 {
                        panic!("Invalid S-Expression, missing binding for let.")
                    }
                    Expr::Let(bind_vec, Box::new(parse_expr(e)))
                },

                // Block statement //
                [Sexp::Atom(S(op)), exprs @ ..] if op == "block" => {
                    let coll:Vec<Expr> = exprs.into_iter().map(parse_expr).collect();
                    if coll.len() == 0 {
                        panic!("Invalid S-Expression")
                    }
                    Expr::Block(coll)
                },

                // set! statement //
                [Sexp::Atom(S(op)), Sexp::Atom(S(name)), e] if op == "set!" => {
                    Expr::Set(name.to_string(), Box::new(parse_expr(e)))
                },

                // Break statement //
                [Sexp::Atom(S(op)), e] if op == "break" => {
                    Expr::Break(Box::new(parse_expr(e)))
                },

                // Empty Argument Function //
                [Sexp::Atom(S(funname))] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")}
                    Expr::Call(funname.to_string())
                },

                // One Argument Function //
                [Sexp::Atom(S(funname)), arg] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")}

                    Expr::Call1(funname.to_string(), Box::new(parse_expr(arg)))
                },
                
                // Two Argument Function //
                [Sexp::Atom(S(funname)), arg1, arg2] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")}
                    Expr::Call2(
                        funname.to_string(),
                        Box::new(parse_expr(arg1)),
                        Box::new(parse_expr(arg2))
                    )
                },
                _ => panic!("Invalid S-Expression."),
            }
        },
        
        
        _ => panic!("Invalid S-Expression.")
        }
}

// PROVIDED LECTURE CODE (https://github.com/ucsd-compilers-s23/lecture1/blob/diamondback/src/main.rs#L334)
fn parse_definition(s: &Sexp) -> (Definition, String) {
    match s {
        Sexp::List(def_vec) => match &def_vec[..] {
            [Sexp::Atom(S(keyword)), Sexp::List(name_vec), body] if keyword == "fun" => match &name_vec[..] {
                [Sexp::Atom(S(funname)), Sexp::Atom(S(arg))] => {
                    if check_reserved_words(funname.clone()) { panic!("Error - keyword used in function defintion.")}
                    (Definition::Fun1(funname.to_string(), arg.to_string(), parse_expr(body)), funname.to_string())
                }
                [Sexp::Atom(S(funname)), Sexp::Atom(S(arg1)), Sexp::Atom(S(arg2))] => {
                    if check_reserved_words(funname.clone()) { panic!("Error - keyword used in function defintion.")}
                    (Definition::Fun2(funname.to_string(), arg1.to_string(), arg2.to_string(), parse_expr(body)), funname.to_string())
                }
                [Sexp::Atom(S(funname))] => {
                    if check_reserved_words(funname.clone()) { panic!("Error - keyword used in function defintion.")}
                    (Definition::Fun(funname.to_string(), parse_expr(body)), funname.to_string())
                }
                _ => panic!("Bad fundef"),

            },
            _ => panic!("Bad fundef"),
        },
        _ => panic!("Bad fundef"),
    }
}

// PROVIDED LECTURE CODE (https://github.com/ucsd-compilers-s23/lecture1/blob/diamondback/src/main.rs#L334)
pub fn parse_program(s: &Sexp) -> Program {
    match s {
        Sexp::List(vec) => {
            let mut defs: Vec<Definition> = vec![];
            let mut func_list = HashSet::new();
            for def_or_exp in vec {
                if is_def(def_or_exp) {
                    let (instr, name) = parse_definition(def_or_exp);
                    defs.push(instr);
                    func_list.insert(name);
                } else {
                    return Program {
                        defs: defs,
                        main: parse_expr(def_or_exp),
                        func_list: func_list,
                    };
                }
            }
            panic!("Only found definitions");
        }
        _ => panic!("Program should be a list")
    }
}

// PROVIDED IN LECTURE CODE (https://github.com/ucsd-compilers-s23/lecture1/blob/diamondback/src/main.rs#L334)
fn is_def(s: &Sexp) -> bool {
    match s {
        Sexp::List(def_vec) => match &def_vec[..] {
            [Sexp::Atom(S(keyword)), Sexp::List(_), _] if keyword == "fun" => true,
            _ => false
        }
        _ => false,
    }
}

// This was inspired by the code from compiler 31 and 17
fn check_reserved_words(name: String) -> bool {
    match &name[..] {
        "let" | "block" | "set!" | "loop" | "break" | "if"   | "input" | "+" |
        "-"   | "*"     | "="    | "true" | "false" | ">"    | "<"     | ">="|
        "<="  | "fun"   | "print"| "sub1" | "add1"  | "isnum"| "isbool" => true,
        _ => false
    }
}

fn parse_bind(s: &Sexp) -> (String, Expr) {


    match s {
        Sexp::List(vec) =>
            match &vec[..] {
                [Sexp::Atom(S(var)), e] => {   
                    if check_reserved_words(var.clone()){
                        panic!("Error - keyword used.")
                    }
                    (String::from(var),parse_expr(e)) },
                _ => panic!("Invalid S-Expression.")
            },
        _ => panic!("Invalid S-Expression.")
    }
}