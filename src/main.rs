mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;

use scanner::Scanner;

use crate::{ast_printer::AstPrinter, interpreter::Interpreter, parser::Parser};

fn main() {
    let casos = vec![
        "1 + 2 * ",
        "10 / 2 - 1",
        "1 + 2 * 3 - 4",
        "(1 + 2 * 3",
        "!true",
        "-42",
        "1 == 1",
        "3 > 2",
    ];

    for caso in casos {
        let mut scanner = Scanner::new(caso.to_string());
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        for error in &parser.errors {
            println!("Erro na linha {}: {}", error.token.line, error.message);
        }

        let mut printer = AstPrinter;
        match expr {
            Ok(e) => {
                println!("{}", e.accept(&mut printer));
                let mut interpreter = Interpreter {};
                match interpreter.evaluate(&e) {
                    Ok(value) => println!("{:?}", value),
                    Err(err) => println!("Erro na linha {}: {}", err.line, err.message),
                }
            }
            Err(_) => {}
        }
    }
}
