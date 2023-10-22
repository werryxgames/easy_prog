<p align="center"><a href="https://github.com/werryxgames/easy_prog/releases" target="blank"><img src="https://github.com/werryxgames/easy_prog/blob/main/icon.png" width="128" alt="Easy Prog logo"></a></p>

# Easy Prog

High-performance dynamicially typed programming language with fully customizable standard library.
Created for using as modifications language or programming language in secure environment.

Developed to be fully predictable. Programs can do absolutely nothing without at least 1 function defined.

## Features

Most useful functions and description of features.

### Lexer (lexical analyzer)

Analyzes source code and converts it to lexical tokens, also helps to recognize errors.

```rust
/// Converts source code to lexical tokens.
pub fn to_tokens(code: &str) -> Result<Vec<Token>, LexerError> {
  ...
}
```

### Parser

Parses input from lexer to AST (Abstract Syntax Tree).

```rust
/// Parses lexical tokens to AST.
pub fn parse_program(tokens: &mut Vector<Token>) -> Result<SequenceNode, ParserError> {
  ...
}
```

```rust
/// Parses source code to AST.
#[cfg(feature = "lexer")]
pub fn parse(code: &str) -> Result<SequenceNode, ParserError> {
  ...
}
```

### Runner

Runs input from parser, modifying scope.

```rust
/// Executes code from AST with given scope.
pub fn execute(scope: &mut Scope, ast: &SequenceNode, path: &str) -> bool {
  ...
}
```

```rust
/// Runs source code with given scope.
#[cfg(feature = "parser")]
pub fn run_code_scope(code: &str, scope: &mut Scope) -> bool {
  ...
}
```

```rust
/// Runs code with default scope.
#[cfg(feature = "parser")]
pub fn run_code(code: &str) -> bool {
  ...
}
```

```rust
/// Runs code from file with given scope.
#[cfg(feature = "parser")]
pub fn run_file_scope(path: &str, scope: &mut Scope) -> bool {
  ...
}
```

```rust
/// Runs code from file with default scope.
#[cfg(feature = "parser")]
pub fn run_file(path: &str) -> bool {
  ...
}
```

### Stdlib

Provides access to I/O, conditional functions, variable and function declaration and more.

```rust
/// Adds default conditional functoins, variable and function declaration and some other functions to specified scope.
pub fn add_core(scope: &mut Scope) {
  ...
}
```

```rust
/// Adds the whole standard library to specified scope.
pub fn add_stdlib(scope: &mut Scope) {
  ...
}
```

### Repl

Read-Eval-Print Loop.

```rust
/// Starts default REPL.
/// Uses rustyline if "repl-rustyline" feature is enabled.
pub fn start_repl() -> ReplError {
  ...
}
```

```rust
/// Starts REPL with specified scope.
#[cfg(feature = "repl-rustyline")]
pub fn start_repl_scope(scope: &mut Scope) -> ReplError {
  ...
}
```

```rust
/// Starts REPL with specified scope.
/// Uses fallback REPL even if "repl-rustyline" feature is enabled.
pub fn start_default_repl<W: Write>(scope: &mut Scope, out: &mut W, in_: Stdin) -> ReplError {
  ...
}
```

### Compiler

Compiles program to and loads it from bytecode.

```rust
todo!()
```

### Translator

Translates Easy Prog program to other programming language.

For example, there is a function `Int add(Int, Int)` that adds two terms and returns result.
It is added to stdlib and another function `int add(int a, int b)` is added to trlib (Translation Library).
Then default C++ file program is generated:
```cpp
#include "easy_prog.h"

...1

int main() {
  ...2
  return 0;
}
```
Then `...1` is replaced with function definitions from trlib, `...2` is replaced with generated program, for example `add(1, add(2, 5))` will be translated to something like
```cpp
#include "easy_prog.h"

Int add(Int a, Int b) {
  return Int(a.number + b.number);
}

int main() {
  add(1, add(2, 5));
  return 0;
}
```
Also function bodies (lambdas) will be created as functions; `if(eq(1, 1), { print("Hello, World!")} )` will be translated to something like
```cpp
#include "easy_prog.h"

Void if(Int cond, Func if_branch) {
  if (cond != 0) {
    if_branch();
  }
  return Void();
}

Int eq(Int a, Int b) {
  return Int((Int_t)(a.number == b.numer));
}

Void print(...);

Variant _unnamed0(...) {
  std::vector<Variant> args;
  GET_ARGS(args);
  print("Hello, World!");
  return Void::variant();
}

int main() {
  if(eq(1, 1), _unnamed0);
  return 0;
}
```

```rust
todo!()
```
