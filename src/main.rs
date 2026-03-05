pub mod ast;
pub mod error;
pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod preprocessor;

use error::LangError;
use evaluator::Environment;
use lexer::TokenType;
use logos::Logos;

fn tokenize(source: &str) -> Result<Vec<TokenType>, LangError> {
    let mut lexer = TokenType::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => tokens.push(token),
            Err(_) => {
                return Err(LangError::lex(format!(
                    "Unexpected token '{}'",
                    lexer.slice()
                )));
            }
        }
    }

    Ok(tokens)
}

fn run(source: &str) -> Result<(), LangError> {
    let processed = preprocessor::preprocess(source);
    let tokens = tokenize(&processed)?;

    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse()?;

    let mut env = Environment::new();
    for stmt in &statements {
        match evaluator::evaluate_stmt(stmt, &mut env)? {
            Some(val) => println!("{}", val),
            None => {}
        }
    }

    Ok(())
}

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
        for i in 0..3:
            for j in 0..3:
                print j
    "#;

    
    println!("{}", source_code);
    

    if let Err(e) = run(source_code) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
