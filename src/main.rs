mod scanner;
mod token;

use scanner::Scanner;

fn main() {
    let casos = vec![
        "var nome = \"Maria\"",               // caso normal
        "var x = 42",                         // número
        "var y = 3.14",                       // decimal
        "if (x == 42) { }",                   // keywords e operadores
        "var z = \"sem fechar",               // string não terminada
        "var 1nome = 10",                     // número seguido de letra
        "var a = @",                          // caractere desconhecido
        "// isso é um comentário\nvar b = 1", // comentário
        "var ação = 1",
        "var médio = 3.14",
    ];

    for caso in casos {
        println!("\n--- Testando: {} ---", caso);
        let mut scanner = Scanner::new(caso.to_string());
        let tokens = scanner.scan_tokens();
        for token in tokens {
            println!(
                "{:?} | lexeme: '{}' | linha: {}",
                token.token_type, token.lexeme, token.line
            );
        }
    }
}
