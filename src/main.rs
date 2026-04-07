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
    var a = \"global\";
    {
        var a = \"local\";
        print a;
        {
            var a = \"sla bicho\";
            print a;
            print a + a;
        }
        print \"chegou aqui\";
        print a;
    }
    print a;
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

    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements);
}
