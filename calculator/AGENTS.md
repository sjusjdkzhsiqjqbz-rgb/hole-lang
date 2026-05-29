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
| `strUncons` | `String -> Option (Char, String)` | Pop first char, returns rest |
| `charEq` | `Char -> Char -> Bool` | Char equality |
| `charIsDigit` | `Char -> Bool` | Is '0'..'9'? |
| `charIsSpace` | `Char -> Bool` | Is whitespace? |
| `charToInt` | `Char -> Int` | Digit char to int (e.g. '7'→7) |
| `intToChar` | `Int -> Char` | Int 0-9 to digit char |
| `strFromList` | `List Char -> String` | Convert char list to string |

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

Use `strUncons` to pop characters from the input string one at a time. Use `charIsDigit` to accumulate digit sequences into numbers. Use `charEq` to detect operators. Use `charToInt` and `intToChar` for digit↔int conversion. Use `strFromList` to convert a `List Char` back to `String` for printing.

```haskell
-- example: check if first char is '+'
isPlus s :: String -> Bool =
  match strUncons s with
    None   -> false
    Some t -> charEq '+' (fst t)
```

Since `Option` is not built-in, define your own:

```haskell
type Option a = None | Some a
type List a = Nil | Cons a (List a)
```

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
