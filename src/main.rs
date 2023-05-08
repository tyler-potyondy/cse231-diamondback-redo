use std::env;
use std::fs::File;
use std::io::prelude::*;

mod types;
mod parser;
mod compiler;
use im::HashMap;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let si = 2;
    let mut l  = 0;

    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let parse_res = sexp::parse(&in_contents);
    let expr_inp = match parse_res {
        Ok(val) => val,
        Err(_) => panic!("Invalid S-Expression.")
    };
    let expr = parser::parse_expr(&expr_inp);
    let start_env = HashMap::new();
    let result = compiler::compile(&expr,si,&start_env, &mut l, &String::from(""));
    let asm_program = format!(
        "
section .text
extern snek_error
extern snek_print
global our_code_starts_here
throw_error:
    push rsp
    call snek_error
our_code_starts_here:
    {}
    ret
overflow:
    mov rdi, {}
    jmp throw_error
invalid_arg:
    mov rdi, {}
    jmp throw_error
",
        result, types::OVERFLOW_ERROR_CODE, types::INVALID_ARGUMENT_ERROR_CODE
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}
