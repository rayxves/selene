mod ast_printer;
mod expr;
mod parser;
mod scanner;
mod token;

use scanner::Scanner;

use crate::{ast_printer::AstPrinter, parser::Parser};

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
        println!("\n--- Testando: {} ---", caso);
        let mut scanner = Scanner::new(caso.to_string());
        let tokens = scanner.scan_tokens();

        for token in &tokens {
            println!(
                "{:?} | lexeme: '{}' | linha: {}",
                token.token_type, token.lexeme, token.line
            );
        }

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        for error in &parser.errors {
            println!("Erro na linha {}: {}", error.token.line, error.message);
        }

        let mut printer = AstPrinter;
        match expr {
            Ok(e) => println!("{}", e.accept(&mut printer)),
            Err(_) => {} 
        }
    }
}
