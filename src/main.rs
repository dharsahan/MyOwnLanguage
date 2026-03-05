pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod preprocessor;
pub mod semantic;
pub mod evaluator;
use lexer::TokenType;
use logos::Logos;
use evaluator::Environment;

fn main() {
    let source_code = r#"
        # Python-like syntax!

        # While loop: count from 1 to 5
        declare i = 1
        while i <= 5:
            print i
            i = i + 1

        # For loop: iterate 0 to 4
        print "--- for loop ---"
        for x in 0..5:
            print x

        # For loop with computation
        declare sum = 0
        for n in 1..11:
            sum = sum + n
        print "Sum 1..10:"
        print sum

        # Nested if/else
        declare val = 15
        if val > 20:
            print "big"
        else if val > 10:
            print "medium"
        else:
            print "small"
    "#;

    // Preprocess: convert indentation + colons → braces
    let processed = preprocessor::preprocess(source_code);
    println!("=== Source ===");
    println!("{}", source_code);
    println!("=== Running ===");

    let mut lexer = TokenType::lexer(&processed);
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
