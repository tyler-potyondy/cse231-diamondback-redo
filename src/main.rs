use std::env;
use std::fs::File;
use std::io::prelude::*;

mod types;
mod parser;
mod compiler;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let prog = "(".to_owned() + &in_contents + ")";
    let parse_res = sexp::parse(&prog);
    let expr_inp = match parse_res {
        Ok(val) => val,
        Err(_) => panic!("Invalid S-Expression.")
    };
    let p = parser::parse_program(&expr_inp);
    let (func_defs, main) = compiler::compile(&p);
    let asm_program = format!(
        "
section .text
extern snek_error
extern snek_print
global our_code_starts_here
throw_error:
    push rsp
    call snek_error
{func_defs}
our_code_starts_here:
    {main}
    ret
overflow:
    mov rdi, {}
    jmp throw_error
invalid_arg:
    mov rdi, {}
    jmp throw_error
",
        types::OVERFLOW_ERROR_CODE, types::INVALID_ARGUMENT_ERROR_CODE
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}
