mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;

use scanner::Scanner;

use crate::{interpreter::Interpreter, parser::Parser};

fn main() {
    let programa = "
        var x = 10;
        var y = 20;
        print x + y;
        print x * 2;
        var nome = \"selene\";
        print nome;
    ";

    let mut scanner = Scanner::new(programa.to_string());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    for error in &parser.errors {
        println!("Erro na linha {}: {}", error.token.line, error.message);
    }

    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements);
}
