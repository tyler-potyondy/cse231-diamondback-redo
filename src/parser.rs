use super::types;
use im::HashSet;
use im::HashMap;
use sexp::*;
use sexp::Atom::*;

use types::Expr;
use types::Op1;
use types::Op2;
use types::Program;
use types::Definition;


pub fn parse_expr(s: &Sexp, is_def: bool, defs: &HashMap<String,u64>) -> types::Expr {
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
                [Sexp::Atom(S(op)), e] if op == "add1"   => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e, is_def,defs))),

                // sub1 operator //
                [Sexp::Atom(S(op)), e] if op == "sub1"   => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e, is_def,defs))),

                // isnum operator //
                [Sexp::Atom(S(op)), e] if op == "isnum"  => Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e, is_def,defs))),

                // isbool operator //
                [Sexp::Atom(S(op)), e] if op == "isbool" => Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e, is_def,defs))),

                // print statement //
                [Sexp::Atom(S(op)), e] if op == "print"  => {
                    Expr::UnOp(Op1::Print, Box::new(parse_expr(e, is_def,defs))) 
                },

                // addition operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "+" => 
                    Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),

                // subtraction operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => 
                    Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),

                // multiplication operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => 
                    Expr::BinOp(Op2::Times, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),

                // Equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "=" => 
                    Expr::BinOp(Op2::Equal, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),

                // Greater than or equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == ">=" => 
                    Expr::BinOp(Op2::GreaterEqual, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),

                // Less than or equal operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "<=" => {
                    Expr::BinOp(Op2::LessEqual, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs)))},

                // Greater than operator //
                [Sexp::Atom(S(op)), e1, e2] if op == ">" => 
                    Expr::BinOp(Op2::Greater, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),

                // Less than operator //
                [Sexp::Atom(S(op)), e1, e2] if op == "<" => 
                    Expr::BinOp(Op2::Less, Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs))),
                
                // if statement //
                [Sexp::Atom(S(op)), e1, e2, e3] if op == "if" => 
                    Expr::If(Box::new(parse_expr(e1, is_def,defs)),Box::new(parse_expr(e2, is_def,defs)), Box::new(parse_expr(e3, is_def,defs))),

                // loop statment //
                [Sexp::Atom(S(op)), e] if op == "loop" =>  
                    Expr::Loop(Box::new(parse_expr(e, is_def,defs))),

                // Let statement // 
                [Sexp::Atom(S(op)), Sexp::List(list_vec), e] if op == "let" => {
                    let mut bind_vec = Vec::new();
                    for item in list_vec{
                        bind_vec.push(parse_bind(item,defs))
                    }
                    if bind_vec.len() == 0 {
                        panic!("Invalid S-Expression, missing binding for let.")
                    }
                    Expr::Let(bind_vec, Box::new(parse_expr(e, is_def,defs)))
                },

                // Block statement //
                [Sexp::Atom(S(op)), exprs @ ..] if op == "block" => {
                    let mut coll:Vec<Expr> = Vec::new();
                    for item in exprs {
                        coll.push(parse_expr(item, is_def,defs));
                    }

                    if coll.len() == 0 {
                        panic!("Invalid S-Expression")
                    }
                    Expr::Block(coll)
                },

                // set! statement //
                [Sexp::Atom(S(op)), Sexp::Atom(S(name)), e] if op == "set!" => {
                    Expr::Set(name.to_string(), Box::new(parse_expr(e, is_def,defs)))
                },

                // Break statement //
                [Sexp::Atom(S(op)), e] if op == "break" => {
                    Expr::Break(Box::new(parse_expr(e, is_def,defs)))
                },

                // Function Call//
                [Sexp::Atom(S(funname)), args @ ..] => {
                    if check_reserved_words(funname.clone()) { panic!("Invalid")} 
                    let mut exprs = Vec::new();               

                    if args.len() as u64 != *(defs.get(funname)).unwrap() {
                        panic!("Invalid, function call must match the number of arguments in declared function.")
                    }
                    for item in args {
                        exprs.push(parse_expr(item, is_def,defs))
                    }

                    Expr::Call(funname.clone(), exprs)
                },
                _ => {
                    panic!("Invalid S-Expression.")
                },
            }
        },
        
        
        _ => panic!("Invalid S-Expression.")
        }
}

// PROVIDED LECTURE CODE (https://github.com/ucsd-compilers-s23/lecture1/blob/diamondback/src/main.rs#L334)
fn parse_definition(s: &Sexp, defs:&HashMap<String,u64>) -> (Definition, String) {
    match s {
        Sexp::List(def_vec) => match &def_vec[..] {
            [Sexp::Atom(S(keyword)), Sexp::List(in_name_vec), body] if keyword == "fun" =>  {
                let mut name_vec = in_name_vec.clone();
                let mut arg_vec = Vec::new();
                if name_vec.len() == 0 {
                   panic!("Invalid - Bad fundef")
                }
                let funname = name_vec[0].clone();
                name_vec.remove(0);

                for item in name_vec {
                    match item {
                        Sexp::Atom(S(str_val)) => {
                            if check_reserved_words(str_val.clone()) || check_reserved_words(str_val.clone()) || str_val == "input"
                            { 
                                panic!("Error - Invalid keyword used in function defintion.")
                            }
                            arg_vec.push(str_val.clone());
                        },
                        _ => panic!("Invalid - Bad fundef"),
                    }
                  
                }
                (Definition::Fun(funname.to_string(), arg_vec, parse_expr(body, true,defs)), funname.to_string())

            },
            _ => panic!("Invalid - Bad fundef"),
        },
        _ => panic!("Invalid - Bad fundef"),
    }
}

pub fn find_arg_num(def_arg_num:&mut HashMap<String,u64>,s:&Sexp) {
    match s {
        Sexp::List(def_vec) => match &def_vec[..] {
            [Sexp::Atom(S(keyword)), Sexp::List(in_name_vec), _] if keyword == "fun" =>  {
                let name_vec = in_name_vec.clone();
                if name_vec.len() == 0 {
                   panic!("Invalid - Bad fundef")
                }
                
                def_arg_num.insert(name_vec[0].clone().to_string(), (name_vec.len() - 1) as u64);

            },
            _ => ()
        },
        _ => (),
    
}
}

// PROVIDED LECTURE CODE (https://github.com/ucsd-compilers-s23/lecture1/blob/diamondback/src/main.rs#L334)
pub fn parse_program(s: &Sexp) -> Program {
    match s {
        Sexp::List(vec) => {
            let mut def_arg_num = HashMap::new();
            for def_or_exp in vec {
                find_arg_num(&mut def_arg_num, def_or_exp);
            }
            let mut defs: Vec<Definition> = vec![];
            let mut func_list = HashSet::new();
            for def_or_exp in vec {
                if is_def(def_or_exp) {
                    let (instr, name) = parse_definition(def_or_exp,&def_arg_num);
                    defs.push(instr);
                    func_list.insert(name);
                } else {
                    if defs.len() + 1 != vec.len() {
                        panic!("Invalid function use")
                    }
                    return Program {
                        defs: defs,
                        main: parse_expr(def_or_exp,false, &def_arg_num),
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

fn parse_bind(s: &Sexp, defs: &HashMap<String,u64>) -> (String, Expr) {


    match s {
        Sexp::List(vec) =>
            match &vec[..] {
                [Sexp::Atom(S(var)), e] => {   
                    if check_reserved_words(var.clone()){
                        panic!("Error - keyword used.")
                    }
                    (String::from(var),parse_expr(e, false,defs)) },
                _ => panic!("Invalid S-Expression.")
            },
        _ => panic!("Invalid S-Expression.")
    }
}