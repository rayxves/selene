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
    print \"--- for com var ---\";
    for (var i = 0; i < 3; i = i + 1) {
        print i;
    }

    print \"--- for sem var ---\";
    var j = 0;
    for (j = 0; j < 3; j = j + 1) {
        print j;
    }

    print \"--- for sem inicializacao ---\";
    var k = 0;
    for (; k < 3; k = k + 1) {
        print k;
    }

    print \"--- logicos ---\";
    var a = null or \"funcionou\";
    print a;

    var b = \"esquerda\" or \"direita\";
    print b;

    var c = false and \"nao avalia\";
    print c;
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