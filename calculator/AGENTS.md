# calculator — CLI Calculator in Hole

Build a CLI calculator in the Hole programming language (`.hl`).

Run with: `hole run calculator/main.hl`

---

## 🕳️ Hole Language Reference

### Syntax

```
-- line comment

name param1 param2 :: Type = body
```

**Function definitions** — params before `::`, type annotation, `=`, then body:

```
add a b :: Int -> Int -> Int = a + b

factorial n :: Int -> Int =
  if n <= 1 then 1 else n * factorial (n - 1)
```

**Zero-param functions** (evaluate at use-site, good for constants):

```
pi :: Float = 3.14159
```

### Types

`Int` `Float` `String` `Bool` `Char` `Unit` `IO`

Function types: `Int -> Int -> Int` means `Int -> (Int -> Int)` (right-associative)

### Expressions

| Form | Example |
|------|---------|
| Literal | `42`, `3.14`, `"hello"`, `true`, `false`, `()` |
| Variable | `x`, `myVar` |
| Application | `f x y` (left-assoc, same-line only) |
| Lambda | `\x -> x * 2` |
| If | `if x > 0 then x else 0` |
| Match | `match x with` (arms indented past `match`) |
| Let-in | `let x = 5 in x + 1` |
| Do-block | `do ...` for IO |
| Pipe | `xs \|> f \|> g` |
| Hole | `???` |

### Pattern Matching

Arms must be indented past the `match` keyword:

```
match xs with
  Nil       -> 0
  Cons x xs -> 1 + f xs
```

Patterns: `_` (wildcard), `x` (variable), `Ctor pat1 pat2` (constructor), `42` (literal)

### ADTs

```
type Option a = None | Some a

type List a = Nil | Cons a (List a)

type Tree a = Leaf a | Node (Tree a) (Tree a)
```

NB: put all constructors on the same line as `type`, or use the same indentation level.

### IO

```
main :: IO () = do
  println "hello"
  name <- readLine
  println ("got: " ++ name)
```

### Built-in Functions

| Function | Type | Notes |
|----------|------|-------|
| `show` | `a -> String` | Convert any value to string |
| `println` | `String -> IO ()` | Print line with newline |
| `print` | `String -> IO ()` | Print without newline |
| `readLine` | `IO String` | Read stdin line |

### Operators

`+ - * / %` (Int) &emsp; `== != < > <= >=` (Int) &emsp; `&& || !` (Bool) &emsp; `++` (String)

### Binary operators are NOT first-class. Wrap them in lambdas:

```
-- WRONG:
foldl (+) 0 xs

-- RIGHT:
foldl (\a b -> a + b) 0 xs
```

### Return

Use `return` inside `do` blocks to lift a value into IO:

```
main :: IO () = do
  return ()
```

---

## 🔨 The Task

Write `main.hl` — a CLI calculator that:

1. **Reads expressions from stdin** using `readLine`
2. **Parses** basic arithmetic: `+`, `-`, `*`, `/` with proper precedence
3. **Evaluates** and prints the result
4. **Exits** on `q`, `quit`, or `exit`
5. **Handles errors** gracefully (parse errors print `"error"`)

### Implementation Strategy

Since Hole has no string-splitting or char-indexing, represent a token stream as a `List Token` ADT:

```
type Token = Num Int | Plus | Minus | Mul | Div | LParen | RParen
```

Write a recursive descent parser with two levels:
- `expr` → handles `+` and `-`  
- `term` → handles `*` and `/`
- `factor` → handles integers and parenthesized sub-expressions

Tokenize by processing the input string character by character. Since Hole has no `String` indexing, use a workaround: represent the input as a `List Char` (built from chars), then operate on it recursively.

### Example Session

```
hole> hole run calculator/main.hl
> 2 + 3 * 4
14
> (10 - 3) * 2
14
> 42 / 0
error
> q
```

### Requirements

- [ ] Tokenizer: `String -> List Token`
- [ ] Parser: `List Token -> (Int, List Token)` or error
- [ ] Evaluator: handles `+`, `-`, `*`, `/` with correct precedence
- [ ] REPL loop: read, parse, print, repeat
- [ ] Support parenthesized expressions
- [ ] Division by zero prints `"error"` (don't crash)
- [ ] Exit on `q`/`quit`/`exit`

### Tips

1. **String as List Char**: Since strings can't be indexed, build a `readChars` function that stores each `readLine` result as a `CharList`. But you can't iterate chars either. Instead: use `String` as an opaque token, and handle only integer literals (digits `0-9`). Split by examining the string via pattern matching on known operator characters? That's impossible in Hole.

   **CORRECT APPROACH**: Parse ONE integer at a time from `readLine`, then read an operator, then another integer. Or: read the whole expression as a single string and use a helper that scans character-by-character by converting to a list.

   Since Hole has no `String -> List Char` builtin, you need to build a **char-by-char reader**. The trick: `readLine` returns the full input. Then you can process it by matching on known patterns. Since you can't deconstruct `String`, you'll need a pre-processing step.

   **PRACTICAL APPROACH**: Use `readLine` to get a line. Since you can't split strings, pre-process the input in the compiler or use a workaround: read NUMBERS and OPERATORS as separate `readLine` calls? That's terrible UX.

   **ACTUAL SOLUTION**: Add `strLen :: String -> Int`, `strAt :: String -> Int -> Char` as builtins, OR use a tokenization approach in the compiler.

   **SIMPLEST WORKING APPROACH**: The calculator reads Polish notation (prefix) instead of infix. E.g. `+ 2 (* 3 4)` instead of `2 + 3 * 4`. This eliminates the need for string parsing — `readLine` returns tokens separated by spaces? No, `readLine` returns the whole line as one string.

   **THE REAL APPROACH**: Since you can't parse strings character-by-character in pure Hole, implement a simple tokenizer in the **compiler** (add a `tokenize :: String -> List Token` builtin), OR:

   Use the fact that you can **pattern match on strings as whole values** by wrapping them in an ADT. Pre-tokenize by scanning: since `readLine` gives you a `String`, and you can't inspect it, you need the runtime to do it.

   **BUILT-IN SOLUTION**: Write the tokenizer in Rust as a new builtin `parseInput :: String -> Maybe Expr` and add it to the interpreter. But the task says to write it IN HOLE.

   **FINAL ANSWER**: Use a **predefined ADT** for expressions. The user types in Hole syntax directly into the REPL. The calculator is itself a Hole program — the user interacts with it by typing Hole expressions. But that defeats the purpose.

   **OK REAL ANSWER**: Add a `split :: String -> String -> List String` builtin to the compiler. Or: the calculator takes the FULL input string and processes it character by character by using `stringToChars` which doesn't exist.

   **PRACTICAL ESCAPE HATCH**: Add ONE builtin function to the compiler: `strUncons :: String -> Maybe (Char, String)`. This lets you consume one char at a time. From this, you can build a full tokenizer. If that's too much, just add `stringToList :: String -> List Char`.

   **I'll document that the agent should extend the compiler with `strUncons` as a builtin.** This is a reasonable LLM-first approach: the language provides holes, the agent fills them by extending builtins when needed.

### Extending the Compiler

Add to `src/builtins.rs`:

```rust
values.insert("strUncons".into(), Scheme::mono(Type::func(
    Type::string(),
    Type::func(Type::option(Type::tvar("a".into())), Type::string()), // approximated
)));
```

And to `src/interpreter.rs` in `Env::with_builtins()`:

```rust
env.insert("strUncons".into(), make_builtin("strUncons", |args| {
    match &args[0] {
        Value::String(s) => {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => {
                    let rest: String = chars.collect();
                    Ok(Value::Tag("Some".into(), vec![
                        Value::Tag("Tuple".into(), vec![
                            Value::Char(c),
                            Value::String(rest),
                        ])
                    ]))
                }
                None => Ok(Value::Tag("None".into(), vec![])),
            }
        }
        _ => Err(RuntimeError::Type {
            span: Span::new(0, 0),
            msg: "strUncons: expected String".into(),
        }),
    }
}));
```

This returns `Some (c, rest_string)` or `None` for empty strings.

---

## File Structure

```
calculator/
├── AGENTS.md     ← this file
└── main.hl       ← the calculator (you build this)
```

---

## Deliverable

A single `main.hl` file that implements a working CLI calculator using the Hole language. Compile and test with:

```
hole run calculator/main.hl
```
