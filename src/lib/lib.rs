#![forbid(
    //missing_docs,
    //warnings,
    anonymous_parameters,
    unused_extern_crates,
    unused_import_braces,
    missing_copy_implementations,
    //trivial_casts,
    variant_size_differences,
    missing_debug_implementations,
    trivial_numeric_casts
)]
// Debug trait derivation will show an error if forbidden.
#![deny(unused_qualifications)]
#![deny(clippy::all)]
#![warn(
    // clippy::pedantic,
    clippy::restriction,
    clippy::cognitive_complexity,
    //missing_docs
)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::missing_inline_in_public_items,
    clippy::implicit_return,
    clippy::wildcard_enum_match_arm,
    clippy::cognitive_complexity,
    clippy::module_name_repetitions,
    clippy::print_stdout,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::non_ascii_literal,
    clippy::float_arithmetic
)]

pub mod environment;
pub mod exec;
pub mod js;
pub mod syntax;

use crate::{
    exec::{Executor, Interpreter},
    syntax::{ast::expr::Expr, lexer::Lexer, parser::Parser},
};

use ontio_std::{
    abi::{Sink, ZeroCopySource},
    console,
    prelude::*,
    runtime,
};

fn parser_expr(src: &str) -> Expr {
    let mut lexer = Lexer::new(src);
    lexer.lex().expect("lexing failed");
    let tokens = lexer.tokens;
    Parser::new(tokens).parse_all().expect("parsing failed")
}

/// Execute the code using an existing Interpreter
/// The str is consumed and the state of the Interpreter is changed
pub fn forward(engine: &mut Interpreter, src: &str) -> String {
    // Setup executor
    let expr = parser_expr(src);
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => format!("{}: {}", "Error", v.to_string()),
    }
}

/// Create a clean Interpreter and execute the code
pub fn exec(src: &str) -> String {
    let mut engine: Interpreter = Executor::new();
    forward(&mut engine, src)
}

#[no_mangle]
pub fn invoke() {
    let input = runtime::input();
    let mut source = ZeroCopySource::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"evaluate" => {
            let js = source.read().unwrap();
            sink.write(evaluate(js))
        }
        b"testcase" => sink.write(testcase()),
        _ => panic!("unsupported action!"),
    }

    runtime::ret(sink.bytes())
}

fn testcase() -> String {
    r#"
    [
        [{"method":"evaluate", "param":"string:1+2", "expected":"string:3"},
        ]
    ]
        "#
    .to_string()
}

#[no_mangle]
pub fn evaluate(src: &str) -> String {
    let mut lexer = Lexer::new(&src);
    match lexer.lex() {
        Ok(_v) => (),
        Err(v) => console::debug(&v.to_string()),
    }

    let tokens = lexer.tokens;

    // Setup executor
    let expr: Expr;

    match Parser::new(tokens).parse_all() {
        Ok(v) => {
            expr = v;
        }
        Err(_v) => {
            console::debug("parsing fail");
            return String::from("parsing failed");
        }
    }

    let mut engine: Interpreter = Executor::new();
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => {
            console::debug(&format!("{} {}", "asudihsiu", v.to_string()));
            format!("{}: {}", "error", v.to_string())
        }
    }
}
