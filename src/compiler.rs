use super::types;

use types::Expr;
use types::Instr;
use types::Val;
use types::Op1;
use types::Op2;
use types::Reg;
use types::Program;
use types::Definition;

use im::HashMap;


fn compile_to_instrs(e: &Expr, mut si: i64, env: &HashMap<String,i64>, l: &mut i32, brake: &String) -> Vec<Instr> {
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
                instr.extend(compile_to_instrs(item, si, env, l, brake));
            }
        },

        // Loop //
        Expr::Loop(e) => {
            // create labels
            let startloop = new_label(l, "loop");
            let endloop = new_label(l, "loopend");

            let e_is = compile_to_instrs(e, si, env, l, &endloop);
            instr.push(Instr::Label(Val::Label(startloop.clone())));
            instr.extend(e_is);
            instr.push(Instr::Jmp(Val::Label(startloop.clone())));
            instr.push(Instr::Label(Val::Label(endloop.clone())));
        },

        // Break // 
        Expr::Break(e) => {
            let e_is = compile_to_instrs(e, si, env, l, brake);
            instr.extend(e_is);
            
            if brake == ""{
                panic!("Error - break must be within a loop.")
            }
            instr.push(Instr::Jmp(Val::Label(brake.clone())));
        }
        
        // Set // 
        Expr::Set(name, val) => {
            println!("{:?}",env);
            let res = env.get(name);
            let offset = match res {
                Some(x) => x,
                None => panic!("Unbound variable identifier {name}"),
            };
            instr.extend(compile_to_instrs(val, si, env, l, brake));
            instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, *offset), Val::Reg(Reg::RAX)));
        }

        // If expression //
        Expr::If(cond, e2, e3) => {

            // evaluate expression of conditional and type check
            instr.extend(compile_to_instrs(cond, si, env, l, brake));

            // create labels
            let cond_label = new_label(l, "if");
            let end_label  = new_label(l, "endif");

            // if conditional false, jump to false branch, otherwise it must be true (number or true bool)
            instr.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::Imm(types::FALSE_VAL)));
            instr.push(Instr::JEqual(Val::Label(cond_label.clone())));

            // else branch
            instr.extend(compile_to_instrs(e2, si, env, l, brake));
            instr.push(Instr::Jmp(Val::Label(end_label.clone())));

            // true branch
            instr.push(Instr::Label(Val::Label(cond_label.clone())));
            instr.extend(compile_to_instrs(e3, si+1, env, l, brake));
            instr.push(Instr::Label(Val::Label(end_label.clone())));

        },

        // Uniary Operations //
        Expr::UnOp(op1, subexpr) => {
            match op1 {
                Op1::Add1 => {
                    update_vec_unop(&mut instr, compile_to_instrs(subexpr,si,env, l, brake),
                    Instr::IAdd(Val::Reg(Reg::RAX), 
                    Val::Imm(1 << 1)));
                    instr.push(Instr::OverFlow())
                },
                Op1::Sub1 => {
                    update_vec_unop(&mut instr, compile_to_instrs(subexpr,si,env, l, brake), 
                    Instr::ISub(Val::Reg(Reg::RAX), 
                    Val::Imm(1 << 1)));
                    instr.push(Instr::OverFlow())
                },
                Op1::IsBool => {
                    instr.extend(compile_to_instrs(subexpr, si, env, l, brake));
                    check_bool_type_instr(&mut instr, l);
                }
                Op1::IsNum => {
                    instr.extend(compile_to_instrs(subexpr,si,env,l, brake));
                    check_num_type_instr(&mut instr, l);
                }
                Op1::Print => {
                    let e_is = compile_to_instrs(subexpr, si, env, l, brake);
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
                        &mut instr, compile_to_instrs(subexpr1,si,env, l, brake), 
                        compile_to_instrs(subexpr2,si+1,env, l, brake), 
                        Instr::IAdd(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, si*8)),
                        si,
                    );
                    instr.push(Instr::OverFlow())
                },
                Op2::Minus => {
                    update_vec_binop(
                        &mut instr, compile_to_instrs(subexpr2,si,env,l, brake), 
                        compile_to_instrs(subexpr1,si+1,env,l, brake), 
                        Instr::ISub(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, si*8)),
                        si,
                    );
                    instr.push(Instr::OverFlow())
                },
                Op2::Times => {
                    let ops = compile_to_instrs(subexpr2,si+1,env,l, brake);
                    
                    update_vec_binop(
                        &mut instr, compile_to_instrs(subexpr1,si,env,l, brake), 
                        ops, 
                        Instr::Shr(Val::Reg(Reg::RAX), Val::Imm(1)),
                        si,
                    );
                    instr.push(Instr::IMul(Val::Reg(Reg::RAX),Val::RegOffset(Reg::RSP, si*8)));
                    instr.push(Instr::OverFlow())
                },
                Op2::Equal => {
                        let op1 = compile_to_instrs(subexpr2,si,env,l, brake);
                        let op2 = compile_to_instrs(subexpr1,si+1,env,l, brake);

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
                        compile_to_instrs(&subexpr1, si, env, l, brake), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake), 
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
                        compile_to_instrs(&subexpr1, si, env, l, brake), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake), 
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
                        compile_to_instrs(&subexpr1, si, env, l, brake), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake), 
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
                        compile_to_instrs(&subexpr1, si, env, l, brake), 
                        compile_to_instrs(&subexpr2, si+1, env, l, brake), 
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
                instr.extend(compile_to_instrs(&item.1, si, &nenv, l, brake));
                nenv = nenv.update(key, si*8); 
                instr.push(Instr::IMov(Val::RegOffset(Reg::RSP, si*8), Val::Reg(Reg::RAX)));
                si = si + 1;
            }
            instr.extend(compile_to_instrs(body, si+1, &nenv, l, brake));
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
        
        _ => panic!("UNCOMPLETE")

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
        Instr::Push(val_a) => format!("\npop {}",val_to_str(val_a)),
        Instr::Pop(val_a) => format!("\npush {}",val_to_str(val_a)),
    }
}

fn val_to_str(v: &Val) -> String {
    match v {
        Val::Reg(Reg::RAX) => String::from("rax"),
        Val::Reg(Reg::RBX) => String::from("rbx"),
        Val::Reg(Reg::RSP) => String::from("rsp"),
        Val::Reg(Reg::RDI) => String::from("rdi"),
        Val::Imm(n) => n.to_string(),
        Val::RegOffset(Reg::RSP,n) => format!("[rsp-{}]",n),
        Val::Label(str_val) => format!("{}",str_val),
        _ => panic!("TODO val_to_str")
    }
}

pub fn compile(e: &types::Expr, si: i64, env: &HashMap<String,i64>, l: &mut i32, brake: &String) -> String {
    let instr_vec = compile_to_instrs(e,si,env, l, brake);
    let mut output = String::new();

    for entry in &instr_vec {
        output = [output, instr_to_str(entry)].join("")
    }
    output
}