use super::types;
use im::HashSet;
use sexp::*;
use sexp::Atom::*;

use types::Expr;
use types::Op1;
use types::Op2;
use types::Program;
use types::Definition;


pub fn parse_expr(s: &Sexp, is_def: bool) -> types::Expr {
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
                if var == "input" && is_def {
                    panic!("Invalid - input keyword cannot be in function definition.")
                }
                Expr::Id(String::from(var))
            }
        },
        Sexp::List(vec) => {
            match &vec[..] {
                // add1 operator //
                [Sexp::Atom(S(op)), e] if op == "add1"   => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e, is_def))),

                // sub1 operator //
                [Sexp::Atom(S(op)), e] if op == "sub1"   => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e, is_def))),

                // isnum operator //
                [Sexp::Atom(S(op)), e] if op == "isnum"  => Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e, is_def))),

                // isbool operator //
                [Sexp::Atom(S(op)), e] if op == "isbool" => Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e, is_def))),

                // print statement //
                [Sexp::Atom(S(op)), e] if op == "print"  => {
                    Expr::UnOp(Op1::Print, Box::new(parse_expr(e, is_def))) 
                },

                // addition operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "+" => 
                    Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),

                // subtraction operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => 
                    Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),

                // multiplication operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => 
                    Expr::BinOp(Op2::Times, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),

                // Equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "=" => 
                    Expr::BinOp(Op2::Equal, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),

                // Greater than or equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == ">=" => 
                    Expr::BinOp(Op2::GreaterEqual, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),

                // Less than or equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "<=" => {
                    Expr::BinOp(Op2::LessEqual, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def)))},

                // Greater than operator //
                [Sexp::Atom(S(op)), e1, e2] if op == ">" => 
                    Expr::BinOp(Op2::Greater, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),

                // Less than operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "<" => 
                    Expr::BinOp(Op2::Less, Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def))),
                
                // if statement //
                [Sexp::Atom(S(op)), e1, e2, e3] if op == "if" => 
                    Expr::If(Box::new(parse_expr(e1, is_def)),Box::new(parse_expr(e2, is_def)), Box::new(parse_expr(e3, is_def))),

                // loop statment //
                [Sexp::Atom(S(op)), e] if op == "loop" =>  
                    Expr::Loop(Box::new(parse_expr(e, is_def))),

                // Let statement // 
                [Sexp::Atom(S(op)), Sexp::List(list_vec), e] if op == "let" => {
                    let mut bind_vec = Vec::new();
                    for item in list_vec{
                        bind_vec.push(parse_bind(item))
                    }
                    if bind_vec.len() == 0 {
                        panic!("Invalid S-Expression, missing binding for let.")
                    }
                    Expr::Let(bind_vec, Box::new(parse_expr(e, is_def)))
                },

                // Block statement //
                [Sexp::Atom(S(op)), exprs @ ..] if op == "block" => {
                    let mut coll:Vec<Expr> = Vec::new();
                    for item in exprs {
                        coll.push(parse_expr(item, is_def));
                    }

                    if coll.len() == 0 {
                        panic!("Invalid S-Expression")
                    }
                    Expr::Block(coll)
                },

                // set! statement //
                [Sexp::Atom(S(op)), Sexp::Atom(S(name)), e] if op == "set!" => {
                    Expr::Set(name.to_string(), Box::new(parse_expr(e, is_def)))
                },

                // Break statement //
                [Sexp::Atom(S(op)), e] if op == "break" => {
                    Expr::Break(Box::new(parse_expr(e, is_def)))
                },

                // Empty Argument Function //
                [Sexp::Atom(S(funname))] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")}
                    Expr::Call(funname.to_string())
                },

                // One Argument Function //
                [Sexp::Atom(S(funname)), arg] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")}

                    Expr::Call1(funname.to_string(), Box::new(parse_expr(arg, is_def)))
                },
                
                // Two Argument Function //
                [Sexp::Atom(S(funname)), arg1, arg2] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")}
                    Expr::Call2(
                        funname.to_string(),
                        Box::new(parse_expr(arg1, is_def)),
                        Box::new(parse_expr(arg2, is_def))
                    )
                },
                _ => {
                    if vec.len() > 3 {
                        let vec1 = &vec[0..3];
                        match vec1 {
                            [Sexp::Atom(S(item)), arg1, arg2] => {
                                if check_reserved_words(item.clone()) { panic!("Invalid S-Expression.") }
                                parse_expr(&Sexp::List(vec!(vec[0].clone(),arg1.clone(),arg2.clone())), is_def)
                            }
                            _ => panic!("Invalid S-Expression."),
                        } 
                        
                    }
                    else {panic!("Invalid S-Expression.")}
                },
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
                    if check_reserved_words(funname.clone()) || check_reserved_words(arg.clone()) || arg == "input"
                    { 
                        panic!("Error - Invalid keyword used in function defintion.")
                    }
                    (Definition::Fun1(funname.to_string(), arg.to_string(), parse_expr(body, true)), funname.to_string())
                }
                [Sexp::Atom(S(funname)), Sexp::Atom(S(arg1)), Sexp::Atom(S(arg2))] => {
                    if check_reserved_words(funname.clone()) || check_reserved_words(arg1.clone()) 
                      || check_reserved_words(arg2.clone()) || arg1 == "input" && arg2 == "input"
                    { 
                        panic!("Error - Invalid keyword used in function defintion.")
                    }                    
                    (Definition::Fun2(funname.to_string(), arg1.to_string(), arg2.to_string(), parse_expr(body, true)), funname.to_string())
                }
                
                [Sexp::Atom(S(funname))] => {
                    if check_reserved_words(funname.clone()) { panic!("Error - Invalid keyword used in function defintion.")}
                    (Definition::Fun(funname.to_string(), parse_expr(body, true)), funname.to_string())
                }
                _ => {
                    let vec = name_vec.clone();
                    let out = vec[0..3].to_vec();
                    println!("{:?}",out);
                    parse_definition(&Sexp::List(vec!(Sexp::Atom(S(keyword.clone())),Sexp::List(out),body.clone())))
                },

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
                        main: parse_expr(def_or_exp,false),
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
                    (String::from(var),parse_expr(e, false)) },
                _ => panic!("Invalid S-Expression.")
            },
        _ => panic!("Invalid S-Expression.")
    }
}