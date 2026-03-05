# MyOwnLanguage

A custom programming language , built from scratch in Rust. Source code is preprocessed from indentation-based blocks into brace-delimited blocks, then lexed, parsed, and interpreted.

## Example

```java
# Variable declaration
declare x = 10

# While loop
declare i = 1
while i <= 5:
    print i
    i = i + 1

# For loop with range
for n in 0..5:
    print n

# If / else if / else
if x > 20:
    print "big"
else if x > 5:
    print "medium"
else:
    print "small"

# Expressions and string concatenation
declare greeting = "Hello"
print greeting + " world"

# Nested loops with break/continue
for i in 0..3:
    for j in 0..3:
        if j == 1:
            continue
        print j
```

## Language Features

| Feature | Syntax |
|---|---|
| Variable declaration | `declare name = expr` |
| Assignment | `name = expr` |
| Print | `print expr` |
| If / else if / else | `if cond:` ... `else if cond:` ... `else:` |
| While loop | `while cond:` |
| For loop (range) | `for var in start..end:` |
| Break / Continue | `break`, `continue` |
| Comments | `# comment` |

### Types

| Type | Examples |
|---|---|
| Number (f64) | `42`, `3.14` |
| String | `"hello"` |
| Char | `'c'` |
| Boolean | `true`, `false` |

### Operators

| Category | Operators |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| String concat | `+` (between strings, or string + number/char) |

Operator precedence (low → high):

1. `==`, `!=`
2. `<`, `>`, `<=`, `>=`
3. `+`, `-`
4. `*`, `/`

Parentheses `( )` can override precedence.

## Project Structure

```
src/
├── main.rs            # Entry point, pipeline orchestration (preprocess → lex → parse → evaluate)
├── preprocessor.rs    # Converts indentation + colon syntax into brace-delimited blocks
├── lexer.rs           # Tokenizer powered by the `logos` crate
├── parser.rs          # Recursive-descent parser with Pratt precedence climbing
├── ast.rs             # AST node definitions (Stmt, Expr, Value, BinaryOperator)
├── evaluator.rs       # Tree-walking interpreter with environment and loop control flow
└── error.rs           # Unified error type (LangError) for lex, parse, and runtime errors
```

### Pipeline

```
Source code
        │
        ▼
┌─────────────────┐
│  Preprocessor   │  Converts indentation + ":" into "{" / "}"
└────────┬────────┘
         ▼
┌─────────────────┐
│     Lexer       │  Tokenizes the brace-delimited source (logos)
└────────┬────────┘
         ▼
┌─────────────────┐
│     Parser      │  Builds an AST from the token stream
└────────┬────────┘
         ▼
┌─────────────────┐
│   Evaluator     │  Walks the AST and executes statements
└─────────────────┘
```

### Module Details

#### `preprocessor.rs`

Transforms indentation-based source into brace-delimited source. Tracks an indent stack and emits `{` after block-opening lines (lines ending with `:` that start with `if`, `else if`, `else`, `while`, or `for`). Emits `}` on dedent. Strips comments and blank lines.

**Input:**
```
if x > 0:
    print "yes"
else:
    print "no"
```

**Output:**
```
if x > 0 {
print "yes"
}
else {
print "no"
}
```

#### `lexer.rs`

Uses the [`logos`](https://crates.io/crates/logos) crate for zero-copy tokenization. Produces a flat `Vec<TokenType>` with variants for:

- **Keywords:** `declare`, `if`, `else`, `while`, `for`, `in`, `break`, `continue`, `return`, `func`, `print`, `true`, `false`
- **Literals:** `Identifier(String)`, `Number(f64)`, `StringLiteral(String)`, `CharLiteral(char)`
- **Operators:** `+`, `-`, `*`, `/`, `==`, `!=`, `>=`, `<=`, `>`, `<`, `=`, `..`
- **Delimiters:** `;`, `(`, `)`, `{`, `}`, `,`

Whitespace and `#`-comments are skipped automatically.

#### `parser.rs`

A hand-written recursive-descent parser that consumes `Vec<TokenType>` and produces `Vec<Stmt>`.

- **Expression parsing** uses Pratt precedence climbing (`parse_binary_op` with `min_prec`).
- **Statement parsing** dispatches on the current token to dedicated methods: `parse_let`, `parse_print`, `parse_if`, `parse_while`, `parse_for`, `parse_assignment_or_expr`.
- **Block parsing** expects matching `{` / `}` delimiters.

#### `ast.rs`

Defines the tree structure:

- **`Value`** — runtime values: `Number(f64)`, `Str(String)`, `Char(char)`, `Boolean(bool)`
- **`Stmt`** — statements: `Let`, `Assign`, `Print`, `Expr`, `If`, `While`, `For`, `Break`, `Continue`
- **`Expr`** — expressions: `Number`, `StringLiteral`, `CharLiteral`, `Boolean`, `Variable`, `BinaryOp`
- **`BinaryOperator`** — `Add`, `Subtract`, `Multiply`, `Divide`, `EqualEqual`, `NotEqual`, `Less`, `Greater`, `LessEqual`, `GreaterEqual`

#### `evaluator.rs`

A tree-walking interpreter that maintains an `Environment` (flat `HashMap<String, Value>`) and evaluates statements in order.

- Arithmetic, string concatenation, and comparison operators are supported across compatible types.
- Truthiness: booleans are direct; numbers are truthy if non-zero; strings are truthy if non-empty; chars are always truthy.
- Loop control flow (`break` / `continue`) is implemented via a `LoopSignal` enum propagated through `evaluate_stmt_inner`.

#### `error.rs`

A unified `LangError` enum with three variants:

- `LexError` — invalid token during lexing
- `ParseError` — unexpected token during parsing
- `RuntimeError` — type mismatch, undefined variable, division by zero, etc.

Implements `Display` and `std::error::Error`.

## Build & Run

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test
```

Requires Rust edition 2024 (`rustc` 1.85+).

## Dependencies

| Crate | Purpose |
|---|---|
| [logos](https://crates.io/crates/logos) 0.16 | Fast lexer generation via derive macros |
