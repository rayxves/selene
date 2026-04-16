mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;

use scanner::Scanner;

use crate::{interpreter::Interpreter, parser::Parser, resolver::Resolver, stmt::Statement};

fn main() {
    let programa = "
   var a = \"global\";
{
    function showA() {
        print a;
    }
    showA();
    var a = \"block\";
    showA();
}
";

    let mut scanner = Scanner::new(programa.to_string());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    if !parser.errors.is_empty() {
        for error in &parser.errors {
            println!("Erro na linha {}: {}", error.token.line, error.message);
        }
        return;
    }
    let interpreter = Interpreter::new();
    let mut resolver = Resolver::new(interpreter);
    if let Err(e) = resolver.resolve(&statements) {
        println!("Erro na linha {}: {}", e.line, e.message);
        return;
    }
    let mut interpreter = resolver.into_interpreter();
    interpreter.interpret(statements);
}
