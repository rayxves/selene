mod expr;
mod parser;
mod scanner;
mod token;

use scanner::Scanner;

use crate::{expr::print_tree, parser::Parser};

fn main() {
    let casos = vec![
        "1 + 2 * 3",
        "10 / 2 - 1",
        "1 + 2 * 3 - 4",
        "(1 + 2) * 3",
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

        match expr {
            Ok(e) => print_tree(&e, 0),
            Err(e) => println!("erro: {}", e),
        }
    }
}
