use super::types;

use types::Expr;
use types::Instr;
use types::Val;
use types::Op1;
use types::Op2;
use types::Reg;
use types::Program;
use types::Definition;

use im::{HashMap,HashSet};


fn compile_to_instrs(e: &Expr, mut si: i64, env: &HashMap<String,i64>, l: &mut i32, brake: &String, func_names: &HashSet<String>) -> Vec<Instr> {
    let mut instr = Vec::new();
    match e {
        Expr::Number(n) => {
            instr.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(*n)));
            instr.push(Instr::Shl(Val::Reg(Reg::RAX),Val::Imm(1)));
        },
        Expr::Boolean(val) => {
            if *val { 
                instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::Imm(types::TRUE_VAL)));
            } else {
                instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::Imm(types::FALSE_VAL)));
            }
        }

        // Block //
        Expr::Block(es) => {
            for item in es {
                instr.extend(compile_to_instrs(item, si, env, l, brake, func_names));
            }
        },

        // Loop //
        Expr::Loop(e) => {
            // create labels
            let startloop = new_label(l, "loop");
            let endloop = new_label(l, "loopend");

            let e_is = compile_to_instrs(e, si, env, l, &endloop, func_names);
            instr.push(Instr::Label(Val::Label(startloop.clone())));
            instr.extend(e_is);
            instr.push(Instr::Jmp(Val::Label(startloop.clone())));
            instr.push(Instr::Label(Val::Label(endloop.clone())));
        },

        // Break // 
        Expr::Break(e) => {
            let e_is = compile_to_instrs(e, si, env, l, brake, func_names);
            instr.extend(e_is);
            
            if brake == ""{
                panic!("Error - break must be within a loop.")
            }
            instr.push(Instr::Jmp(Val::Label(brake.clone())));
        }
        
        // Set // 
        Expr::Set(name, val) => {
            let res = env.get(name);
            let offset = match res {
                Some(x) => x,
                None => panic!("Unbound variable identifier {name}"),
            };
            instr.extend(compile_to_instrs(val, si, env, l, brake, func_names));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, *offset), Val::Reg(Reg::RAX)));
        }

        // If expression //
        Expr::If(cond, e2, e3) => {

            // evaluate expression of conditional and type check
            instr.extend(compile_to_instrs(cond, si, env, l, brake, func_names));

            // create labels
            let cond_label = new_label(l, "if");
            let end_label  = new_label(l, "endif");

            // if conditional false, jump to false branch, otherwise it must be true (number or true bool)
            instr.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::Imm(types::FALSE_VAL)));
            instr.push(Instr::JEqual(Val::Label(cond_label.clone())));

            // else branch
            instr.extend(compile_to_instrs(e2, si, env, l, brake, func_names));
            instr.push(Instr::Jmp(Val::Label(end_label.clone())));

            // true branch
            instr.push(Instr::Label(Val::Label(cond_label.clone())));
            instr.extend(compile_to_instrs(e3, si+1, env, l, brake, func_names));
            instr.push(Instr::Label(Val::Label(end_label.clone())));

        },

        // Uniary Operations //
        Expr::UnOp(op1, subexpr) => {
            match op1 {
                Op1::Add1 => {
                    update_vec_unop(&mut instr, compile_to_instrs(subexpr,si,env, l, brake, func_names),
                    Instr::IAdd(Val::Reg(Reg::RAX), 
                    Val::Imm(1 << 1)));
                    instr.push(Instr::OverFlow())
                },
                Op1::Sub1 => {
                    update_vec_unop(&mut instr, compile_to_instrs(subexpr,si,env, l, brake, func_names), 
                    Instr::ISub(Val::Reg(Reg::RAX), 
                    Val::Imm(1 << 1)));
                    instr.push(Instr::OverFlow())
                },
                Op1::IsBool => {
                    instr.extend(compile_to_instrs(subexpr, si, env, l, brake, func_names));
                    check_bool_type_instr(&mut instr, l);
                }
                Op1::IsNum => {
                    instr.extend(compile_to_instrs(subexpr,si,env,l, brake, func_names));
                    check_num_type_instr(&mut instr, l);
                }
                Op1::Print => {
                    let e_is = compile_to_instrs(subexpr, si, env, l, brake, func_names);
                    let index:i64 = if si % 2 == 1 { si + 1 } else { si };
                    let offset = (index * 8) as u64;
                    instr.extend(e_is);
                    instr.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm(offset)));
                    instr.push(Instr::Push(Val::Reg(Reg::RDI)));
                    instr.push(Instr::IMov(Val::Reg(Reg::RDI),Val::Reg(Reg::RAX)));
                    instr.push(Instr::Call(Val::Label(String::from("snek_print"))));
                    instr.push(Instr::Pop(Val::Reg(Reg::RDI)));
                    instr.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(offset)));
                }
          }
        },
        
        // Binary Operations //
        Expr::BinOp(op2,subexpr1, subexpr2) => {
            match op2 {
                Op2::Plus => {
                    update_vec_binop(
                        &mut instr, compile_to_instrs(subexpr1,si,env, l, brake, func_names), 
                        compile_to_instrs(subexpr2,si+1,env, l, brake, func_names), 
                        Instr::IAdd(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, si*8)),
                        si,
                    );
                    instr.push(Instr::OverFlow())
                },
                Op2::Minus => {
                    update_vec_binop(
                        &mut instr, compile_to_instrs(subexpr2,si,env,l, brake, func_names), 
                        compile_to_instrs(subexpr1,si+1,env,l, brake, func_names), 
                        Instr::ISub(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, si*8)),
                        si,
                    );
                    instr.push(Instr::OverFlow())
                },
                Op2::Times => {
                    let ops = compile_to_instrs(subexpr2,si+1,env,l, brake, func_names);
                    
                    update_vec_binop(
                        &mut instr, compile_to_instrs(subexpr1,si,env,l, brake, func_names), 
                        ops, 
                        Instr::Shr(Val::Reg(Reg::RAX), Val::Imm(1)),
                        si,
                    );
                    instr.push(Instr::IMul(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, si*8)));
                    instr.push(Instr::OverFlow())
                },
                Op2::Equal => {
                        let op1 = compile_to_instrs(subexpr2,si,env,l, brake, func_names);
                        let op2 = compile_to_instrs(subexpr1,si+1,env,l, brake, func_names);

                        let offset = si*8;

                        instr.extend(op1);
                        instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, offset), Val::Reg(Reg::RAX)));
                        instr.extend(op2);

                        // check that both expressions are of same type
                        same_type_expr(&mut instr, offset);

                        // compare values for equivalence
                        instr.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                        instr.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Imm(3)));
                        instr.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
                        instr.push(Instr::Cmove(Val::Reg(Reg::RAX),Val::Reg(Reg::RBX)));
                    },
                Op2::Greater => {
                    compare_size(&mut instr, 
                        compile_to_instrs(&subexpr1, si, env, l, brake, func_names), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake, func_names), 
                        si);

                    // create labels
                    let cond_label = new_label(l, "if");
                    let end_label  = new_label(l, "endif");

                    // jump to if condition if greater than
                    instr.push(Instr::JGreater(Val::Label(cond_label.clone())));
                    
                    conditional_jmp_compare(&mut instr, end_label, cond_label);

                },
                Op2::GreaterEqual => {
                    compare_size(&mut instr, 
                        compile_to_instrs(&subexpr1, si, env, l, brake, func_names), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake, func_names), 
                        si);

                    // create labels
                    let cond_label = new_label(l, "if");
                    let end_label  = new_label(l, "endif");

                    // jump to if condition if greater than or equal
                    instr.push(Instr::JGreaterEqual(Val::Label(cond_label.clone())));
                    
                    conditional_jmp_compare(&mut instr, end_label, cond_label);

                },
                Op2::Less => {
                    compare_size(&mut instr, 
                        compile_to_instrs(&subexpr1, si, env, l, brake, func_names), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake, func_names), 
                        si);

                    // create labels
                    let cond_label = new_label(l, "if");
                    let end_label  = new_label(l, "endif");

                    // jump to if condition if less than
                    instr.push(Instr::JLess(Val::Label(cond_label.clone())));
                    
                    conditional_jmp_compare(&mut instr, end_label, cond_label);

                },
                Op2::LessEqual => {
                    compare_size(&mut instr, 
                        compile_to_instrs(&subexpr1, si, env, l, brake, func_names), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake, func_names), 
                        si);

                    // create labels
                    let cond_label = new_label(l, "if");
                    let end_label  = new_label(l, "endif");

                    // jump to if condition if less than or equal
                    instr.push(Instr::JLessEqual(Val::Label(cond_label.clone())));
                    
                    conditional_jmp_compare(&mut instr, end_label, cond_label);

                },
            }
        },

        // Let Expression //
        Expr::Let(vec,body) => {
            let mut nenv = env.clone();
            let mut scope_keys:std::collections::HashSet<String> = std::collections::HashSet::new();
            for item in vec {
                let key = item.0.clone();
                if scope_keys.contains(&key) {
                    panic!("Error - Duplicate binding.")
                } else {
                    scope_keys.insert(key.clone());
                }
                instr.extend(compile_to_instrs(&item.1, si, &nenv, l, brake, func_names));
                nenv = nenv.update(key, si*8); 
                instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, si*8), Val::Reg(Reg::RAX)));
                si = si + 1;
            }
            instr.extend(compile_to_instrs(body, si+1, &nenv, l, brake, func_names));
        },

        // Variable string //
        Expr::Id(s) => {if s == "input" {        
            instr.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Reg(Reg::RDI)));
            instr.push(Instr::OverFlow())
        }
        else {
            let output = env.get(s);
            match output {
                Option::Some(x) => instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, *x))),
                Option::None => {
                    let s_out = &s[..];
                    panic!("Error - Unbound variable identifier {}",s_out);
                }
           }
        }},

        Expr::Call(name) => {
            if ! func_names.contains(name) {
                panic!("Error - Invalid function call without definition.")
            }
            instr.push(Instr::Jmp(Val::Label(name.clone())));
        },

        Expr::Call1(name, arg) => {
            if ! func_names.contains(name) {
                panic!("Error - Invalid function call without definition.")
            }
            let arg_instrs = compile_to_instrs(arg, si, env, l, brake, func_names);
            let offset = ((si*8) + 8) as u64;

            // instr.extend(arg_instrs);
            // instr.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm(offset)));
            // instr.push(Instr::Push(Val::Reg(Reg::RDI)));
            // instr.push(Instr::IMov(Val::Reg(Reg::RDI),Val::Reg(Reg::RAX)));
            // instr.push(Instr::Call(Val::Label(String::from(name))));
            // instr.push(Instr::Pop(Val::Reg(Reg::RDI)));
            // instr.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(offset)));

            instr.extend(arg_instrs);
            instr.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm(offset)));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, 0), Val::Reg(Reg::RAX)));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, -8), Val::Reg(Reg::RDI)));
            instr.push(Instr::Call(Val::Label(name.clone())));
            instr.push(Instr::IMov(Val::Reg(Reg::RDI),Val::RegOffset(Reg::RSP, -8)));
            instr.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(offset)));

        },
        Expr::Call2(name, arg1, arg2) => {
            let arg_1_instrs = compile_to_instrs(arg1, si, env, l, brake, func_names);
            let arg_2_instrs = compile_to_instrs(arg2, si+1, env, l, brake, func_names);

            let curr_word = si*8;
            let offset = ((si*8) + (2*8)) as u64;
            if ! func_names.contains(name) {
                panic!("Error - Invalid function call without definition.")
            }

            instr.extend(arg_1_instrs);
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, curr_word), Val::Reg(Reg::RAX)));
            instr.extend(arg_2_instrs);

            instr.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm(offset)));

            instr.push(Instr::IMov(Val::Reg(Reg::RBX), Val::RegOffset(Reg::RSP, -16)));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, 0), Val::Reg(Reg::RBX)));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, -8), Val::Reg(Reg::RAX)));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, -16), Val::Reg(Reg::RDI)));

            instr.push(Instr::Call(Val::Label(name.clone())));
            instr.push(Instr::IMov(Val::Reg(Reg::RDI),Val::RegOffset(Reg::RSP, -16)));
            instr.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(offset)));



        },
        

    }
    instr
}


fn conditional_jmp_compare(instr: &mut Vec<Instr>, end_label: String, cond_label: String){
    // condition not met, set RAX to false
    instr.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(types::FALSE_VAL)));
    instr.push(Instr::Jmp(Val::Label(end_label.clone())));

    // condition met, set RAX to true
    instr.push(Instr::Label(Val::Label(cond_label.clone())));
    instr.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(types::TRUE_VAL)));        
    instr.push(Instr::Label(Val::Label(end_label.clone())));
}

fn compare_size(vec: &mut Vec<Instr>, e1: Vec<Instr>, e2: Vec<Instr>, si: i64){
    // compute expression1, save to memory
    vec.extend(e1);
    vec.push(Instr::IMov(Val::RegOffset(Reg::RSP, si*8), Val::Reg(Reg::RAX)));

    // compute expression2 
    vec.extend(e2);

    // confirm that they are the same type 
    same_type_expr(vec, si *8);

    // confirm that of type number
    type_number_check(vec);

    // compare values
    vec.push(Instr::Cmp(Val::RegOffset(Reg::RSP, si*8), Val::Reg(Reg::RAX)));    

}

fn type_number_check(vec: &mut Vec<Instr>){
    vec.push(Instr::Test(Val::Reg(Reg::RAX),Val::Imm(1)));
    vec.push(Instr::JNotEqual(Val::Label(String::from("invalid_arg"))));
}


fn update_vec_binop(vec: &mut Vec<Instr>, append1: Vec<Instr>, append2: Vec<Instr>, append3: Instr, si: i64) {
    vec.extend(append1);
    let stack_offset = si * 8;

    // confirm that value is a number 
    type_number_check(vec);

    vec.push(Instr::IMov(Val::RegOffset(Reg::RSP, stack_offset),Val::Reg(Reg::RAX)));
    vec.extend(append2);

    // confirm that value is a number 
    type_number_check(vec);
    vec.push(append3);
}

fn update_vec_unop(vec: &mut Vec<Instr>, append1: Vec<Instr>, append2: Instr) {
    vec.extend(append1);
    type_number_check(vec);
    vec.push(append2);
}

fn new_label(l: &mut i32, s: &str) -> String {
    let current = *l;
    *l += 1;
    format!("{s}_{current}")
}


fn check_bool_type_instr(instr: &mut Vec<Instr>, l: &mut i32){
    // Create labels
    let cond_label = new_label(l, "if");
    let end_label  = new_label(l, "endif");

    // bitwise and with RAX register and 1 to determine if LSB is 0
    // LSB of 0 denotes that the value is a number -> jmp if not zero...i.e. true if bool
    instr.push(Instr::Test(Val::Reg(Reg::RAX),Val::Imm(1)));
    instr.push(Instr::JNotEqual(Val::Label(cond_label.clone())));

    // else branch --> value is not a bool
    instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::Imm(types::FALSE_VAL)));
    instr.push(Instr::Jmp(Val::Label(end_label.clone())));

    // if true branch --> value is a bool
    instr.push(Instr::Label(Val::Label(cond_label.clone())));
    instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::Imm(types::TRUE_VAL)));
    instr.push(Instr::Label(Val::Label(end_label.clone())))
}

fn check_num_type_instr(instr: &mut Vec<Instr>, l: &mut i32){
    // Create labels
    let cond_label = new_label(l, "if");
    let end_label  = new_label(l, "endif");

    // bitwise and with RAX register and 1 to determine if LSB is 0
    // LSB of 0 denotes that the value is a number -> jmp if not zero...i.e. true if bool
    instr.push(Instr::Test(Val::Reg(Reg::RAX),Val::Imm(1)));
    instr.push(Instr::JNotEqual(Val::Label(cond_label.clone())));
    
    // else branch --> value is a number 
    instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::Imm(types::TRUE_VAL)));
    instr.push(Instr::Jmp(Val::Label(end_label.clone())));

    // if true branch --> value is not a number
    instr.push(Instr::Label(Val::Label(cond_label.clone())));
    instr.push(Instr::IMov(Val::Reg(Reg::RAX),Val::Imm(types::FALSE_VAL)));
    instr.push(Instr::Label(Val::Label(end_label.clone())))
}

fn same_type_expr(instr: &mut Vec<Instr>, offset:i64){
    instr.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)));
    instr.push(Instr::Xor(Val::Reg(Reg::RBX),Val::RegOffset(Reg::RSP, offset)));

    instr.push(Instr::Test(Val::Reg(Reg::RBX), Val::Imm(1)));

    // jmp if error case met
    instr.push(Instr::JNotEqual(Val::Label(String::from("invalid_arg"))));
}

fn instr_to_str(i: &Instr) -> String {
    match i {
        Instr::IMov(val_a, val_b) => format!("\nmov {}, {}", val_to_str(val_a), val_to_str(val_b)),
        Instr::IAdd(val_a, val_b) => format!("\nadd {}, {}", val_to_str(val_a), val_to_str(val_b)),
        Instr::ISub(val_a, val_b) => format!("\nsub {}, {}", val_to_str(val_a), val_to_str(val_b)),
        Instr::IMul(val_a, val_b) => format!("\nimul {}, {}", val_to_str(val_a), val_to_str(val_b)),
        Instr::Shr(val_a, val_b) => format!("\nsar {},{}", val_to_str(val_a), val_to_str(val_b)),
        Instr::Shl(val_a,val_b) => format!("\nshl {},{}",val_to_str(val_a),val_to_str(val_b)),
        Instr::Cmp(val_a, val_b) => format!("\ncmp {},{}", val_to_str(val_a), val_to_str(val_b)),
        Instr::JEqual(val_a) => format!("\nje {}", val_to_str(val_a)),
        Instr::Jmp(val_a) => format!("\njmp {}", val_to_str(val_a)),
        Instr::JNotEqual(val_a) => format!("\njne {}", val_to_str(val_a)),
        Instr::JGreater(val_a) => format!("\njg {}", val_to_str(val_a)),
        Instr::JGreaterEqual(val_a) => format!("\njge {}", val_to_str(val_a)),
        Instr::JLess(val_a) => format!("\njl {}", val_to_str(val_a)),
        Instr::JLessEqual(val_a) => format!("\njle {}", val_to_str(val_a)),
        Instr::Test(val_a, val_b) => format!("\ntest {},{}", val_to_str(val_a), val_to_str(val_b)),
        Instr::Label(val_a) => format!("\n{}:",val_to_str(val_a)),
        Instr::Xor(val_a,val_b) => format!("\nxor {},{}",val_to_str(val_a),val_to_str(val_b)),
        Instr::Cmove(val_a,val_b) => format!("\ncmove {},{}",val_to_str(val_a),val_to_str(val_b)),
        Instr::OverFlow() => format!("\njo overflow"),
        Instr::Call(val_a) => format!("\ncall {}", val_to_str(val_a)),
        Instr::Push(val_a) => format!("\npush {}",val_to_str(val_a)),
        Instr::Pop(val_a) => format!("\npop {}",val_to_str(val_a)),
        Instr::Ret() => format!("\nret"),
    }
}

fn val_to_str(v: &Val) -> String {
    match v {
        Val::Reg(Reg::RAX) => String::from("rax"),
        Val::Reg(Reg::RBX) => String::from("rbx"),
        Val::Reg(Reg::RSP) => String::from("rsp"),
        Val::Reg(Reg::RDI) => String::from("rdi"),
        Val::Imm(n) => n.to_string(),
        Val::RegOffset(Reg::RSP,n) => {
            if *n < 0 {
                format!("[rsp+{}]",-1 * n)}
            else {
                format!("[rsp-{}]",n)}
            },
        Val::Label(str_val) => format!("{}",str_val),
        _ => panic!("TODO val_to_str")
    }
}

fn compile_definition_instrs(d: &Definition, labels: &mut i32, func_names: &HashSet<String>) -> Vec<Instr> {
    let (env, body, name) = match d {
        Definition::Fun(name, body) => {
            let body_env:HashMap<String,i64> = HashMap::new();
            (body_env, body, name)
        }

        // store arg1 at RSP + 8 (insert as negative because code generator does RSP - {offset}
        Definition::Fun1(name, arg, body) => {
            let mut body_env:HashMap<String,i64> = HashMap::new();
            body_env.insert(String::from(arg),-8);
            (body_env, body, name)
        }

        // store arg1 and arg2 at RSP + 8 and RSP + 16 (insert as negative because code generator does RSP - {offset}
        Definition::Fun2(name, arg1, arg2, body) => {
            if arg1 == arg2 {
                panic!("Error - invalid function declaration; parameter is declared twice")
            }
            let mut body_env:HashMap<String,i64> = HashMap::new();
            
            body_env.insert(String::from(arg1),-8);
            body_env.insert(String::from(arg2), -16);
            (body_env, body, name)
        }
    };
    let mut out_instrs = Vec::new();

    // add label for function name
    out_instrs.push(Instr::Label(Val::Label(name.clone())));

    // compile instructions for function body
    out_instrs.extend(compile_to_instrs(body, 2, &env, labels, &String::from(""), func_names));
    out_instrs.push(Instr::Ret());
    out_instrs

}

// this function incorporates aspects of the compile_program and compile_definition functions in the lecture code
pub fn compile(p: &Program) -> (String,String) {
    // create empty environment 
    let env:&HashMap<String,i64> = &HashMap::new();

    // initialize stack index, brake string, and label index
    let si = 2;
    let mut labels  = 0;
    let brake = String::from("");

    // create instructions for function defintions
    let mut def_instrs:Vec<Instr> = Vec::new();

    // create hashset to insure each function declaration is unique 
    let mut func_names = HashSet::new();

    for def in &p.defs[..] {
        let name = match def {
            Definition::Fun(name,_) => name,
            Definition::Fun1(name,_ ,_) => name,
            Definition::Fun2(name,_ ,_ ,_) => name,
        };
        if func_names.contains(name) { 
            panic!("Error - invalid function declaration, function {name} declare multiple times.")
        }
        func_names.insert(name.clone());

        def_instrs.extend(compile_definition_instrs(&def, &mut labels, &p.func_list));
      }
    
    // create instructions for main body
    let main_instrs = compile_to_instrs(&p.main,si,env, &mut labels, &brake, &p.func_list);

    let mut def_output = String::new();
    let mut main_output = String::new();

    // convert def instr vector to assembly str
    for entry in &def_instrs {
        def_output = [def_output, instr_to_str(entry)].join("")
    }

    // convert main instr vector to assembly str
    for entry in &main_instrs {
        main_output = [main_output, instr_to_str(entry)].join("")
    }
    (def_output,main_output)

}