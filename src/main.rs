use crate::{lexer::Lexer, parser::Parser};

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

fn main() {
    let src = "
    fun f(x) {
        if x <= 1 return x
        else {
            return f(x - 1) + f(x - 2)
        }
    }
    let x = (1 + 2) * 3
    ";
    let lexer = Lexer::new(src);
    let mut parser = Parser::new(lexer);
    println!("{:?}", parser.parse());
}
