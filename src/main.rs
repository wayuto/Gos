use crate::{
    compiler::Compiler,
    gvm::Gvm,
    lexer::Lexer,
    parser::Parser,
    preprocessor::Preprocessor,
    serialize::{compile, load},
};
use clap::{Arg, Command};
use std::{fs, path::Path};

pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod gvm;
pub mod lexer;
pub mod parser;
pub mod preprocessor;
pub mod serialize;
pub mod token;

fn main() {
    let cmd = Command::new("gos")
        .version("0.2.7#rust")
        .about("Gos interpreter implemented in Rust")
        .arg(Arg::new("FILE").help("Run the Gos source/bytecode file"))
        .arg(
            Arg::new("ast")
                .short('a')
                .long("ast")
                .help("Print AST of the Gos source file"),
        )
        .arg(
            Arg::new("compile")
                .short('c')
                .long("compile")
                .help("Compile the Gos source file"),
        )
        .arg(
            Arg::new("preprocess")
                .short('p')
                .long("preprocess")
                .help("Print the preprocessed Gos source file"),
        )
        .arg(
            Arg::new("disassemble")
                .short('d')
                .long("disassemble")
                .help("Run the Gos source/bytecode file"),
        );

    if std::env::args().len() == 1 {
        cmd.clone().print_help().unwrap();
        std::process::exit(0);
    }

    let matches = cmd.get_matches();

    if let Some(file) = matches.get_one::<String>("FILE") {
        let ext = Path::new(file)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        match ext.as_deref() {
            Some("gbc") => {
                let bytecode = load(file.to_string());
                let mut gvm = Gvm::new(bytecode);
                gvm.run();
            }
            _ => {
                let src = fs::read_to_string(file).unwrap();
                let path = Path::new(&file.clone())
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let mut preprocessor = Preprocessor::new(src.as_str(), path);
                let code = preprocessor.preprocess();
                let lexer = Lexer::new(code.as_str());
                let mut parser = Parser::new(lexer);
                let ast = parser.parse();
                let mut compiler = Compiler::new();
                let bytecode = compiler.compile(ast);
                let mut gvm = Gvm::new(bytecode);
                gvm.run();
            }
        }
    } else if let Some(file) = matches.get_one::<String>("ast") {
        let src = fs::read_to_string(file).unwrap();
        let path = Path::new(&file.clone())
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut preprocessor = Preprocessor::new(src.as_str(), path);
        let code = preprocessor.preprocess();
        let lexer = Lexer::new(code.as_str());
        let mut parser = Parser::new(lexer);
        let ast = parser.parse();
        println!("{:#?}", ast);
    } else if let Some(file) = matches.get_one::<String>("preprocess") {
        let src = fs::read_to_string(file).unwrap();
        let path = Path::new(&file.clone())
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut preprocessor = Preprocessor::new(src.as_str(), path);
        let code = preprocessor.preprocess();
        println!("{}", code);
    } else if let Some(file) = matches.get_one::<String>("compile") {
        compile(file.to_string());
    } else if let Some(file) = matches.get_one::<String>("disassemble") {
        let ext = Path::new(file)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        match ext.as_deref() {
            Some("gbc") => {
                load(file.to_string()).print();
            }
            _ => {
                let src = fs::read_to_string(file).unwrap();
                let path = Path::new(&file.clone())
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let mut preprocessor = Preprocessor::new(src.as_str(), path);
                let code = preprocessor.preprocess();
                let lexer = Lexer::new(code.as_str());
                let mut parser = Parser::new(lexer);
                let ast = parser.parse();
                let mut compiler = Compiler::new();
                let bytecode = compiler.compile(ast);
                bytecode.print();
            }
        }
    }
}
