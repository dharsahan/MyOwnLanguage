pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod evaluator;
use lexer::TokenType;
use logos::Logos;
use evaluator::Environment;

fn main() {
    let source_code = r#"
        declare x = 10
        declare y = 20
        if x + y == 30{
            print "x + y is 30"
        } else {
            print "x + y is not 30"
        }
    "#;           


    println!("{}", source_code);

    let mut lexer = TokenType::lexer(source_code);
    let mut tokens = Vec::new();

    while let Some(token_res) = lexer.next() {
        if let Ok(token) = token_res {
            tokens.push(token);
        } else {
            eprintln!("Lexer error at '{}'", lexer.slice());
            return;
        }
    }

    let mut parser = parser::Parser::new(tokens);
    match parser.parse() {
        Ok(statements) => {
            let mut env = Environment::new();
            for stmt in statements {
                match evaluator::evaluate_stmt(&stmt, &mut env) {
                    Ok(Some(val)) => println!("{}", val),
                    Ok(None) => {}, 
                    Err(e) => eprintln!("Runtime Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Parse Error: {}", e),
    }
}
