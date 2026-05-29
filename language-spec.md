# Hole Language Specification

**Hole** is an experimental LLM-first functional programming language.  
Extension: `.hl` | Paradigm: pure functional, eagerly evaluated | Typing: static, Hindley-Milner inference

## Design Philosophy

Hole optimizes for **LLMs generating code**, not humans reading it:

| Principle         | Mechanism                                              |
|-------------------|--------------------------------------------------------|
| Explicitness      | No implicit coercion, no null, IO is explicit          |
| Type-driven       | Top-level annotations required — serve as LLM prompts  |
| Holes             | `???` placeholders; compiler reports expected type     |
| Exhaustiveness    | Pattern matches must cover all variants                |
| Purity            | Functions are pure unless wrapped in `IO`              |
| Tool integration  | `--json` mode outputs structured errors for LLM use    |

---

## Lexical Structure

### Comments
```
-- single-line comment
```

### Keywords
`module` `exposing` `type` `alias` `do` `if` `then` `else` `match` `with` `let` `in` `return`

### Operators
Arithmetic: `+` `-` `*` `/` `%`  
Comparison: `==` `!=` `<` `>` `<=` `>=`  
Logical: `&&` `||` `!`  
String: `++`  
Pipe: `|>`

### Special Tokens
`::` (type annotation)  `->` (arrow)  `\` (lambda)  `<-` (do-bind)  `=` (definition)  `???` (hole)  `|` (variant separator)

### Literals
```
42            -- Int
3.14          -- Float
"hello"       -- String
'a'           -- Char
true, false   -- Bool
()            -- Unit
```

---

## Types

### Primitives
`Int` `Float` `String` `Bool` `Char` `Unit` `IO`

### Functions
`Int -> Int -> Int` — right-associative, means `Int -> (Int -> Int)`

### Tuples
`(Int, String, Bool)`

### Algebraic Data Types
```
type Option a = None | Some a

type List a = Nil | Cons a (List a)

type Tree a = Leaf a | Node (Tree a) (Tree a)
```

### Type Aliases
```
type alias Point = (Float, Float)
```

---

## Expressions

### Literals
```
42, true, "hello", ()
```

### Variables
```
x, someVar, list
```

### Function Application
Left-associative, limited to same line:
```
f x y          -- means ((f x) y)
add 1 2
print "hi"
```

### Lambda
```
\x -> x + 1
\x y -> x + y
```

### If Expression
```
if x > 0 then x else -x
```

### Match Expression
Arms must be indented past the `match` keyword. Exhaustiveness enforced by compiler.

```
match xs with
  Nil       -> 0
  Cons x xs -> 1 + length xs
```

### Let Expression
```
let
  x = 10
  y = 20
in
  x + y
```

Let bindings can be functions:
```
let
  double x = x * 2
  square x = x * x
in
  double (square 3)
```

### Do Block (IO)
```
do
  println "Enter name:"
  name <- readLine
  println ("Hello, " ++ name)
  return ()
```

### Pipe Operator
```
xs |> map (\x -> x * 2) |> filter (\x -> x > 10)
```

### List Literal
```
[1, 2, 3]
```

### Tuple Literal
```
(1, "hello", true)
```

### Hole
```
???     -- compiler reports: "hole of type Int"
```

---

## Function Definitions

Syntax: `name params :: Type = body`

```
add a b :: Int -> Int -> Int = a + b

gcd a b :: Int -> Int -> Int =
  if b == 0 then a else gcd b (a % b)

factorial n :: Int -> Int =
  if n <= 1 then 1 else n * factorial (n - 1)

-- zero-parameter function
pi :: Float = 3.14159
```

---

## Modules

```
module Math exposing (add, mul)

add a b :: Int -> Int -> Int = a + b

mul a b :: Int -> Int -> Int = a * b
```

---

## Built-in Functions

| Function    | Type                      |
|-------------|---------------------------|
| `show`      | `a -> String`             |
| `print`     | `String -> IO ()`         |
| `println`   | `String -> IO ()`         |
| `readLine`  | `IO String`               |

Operator builtins (used by parser, not called directly):
`iadd` `isub` `imul` `idiv` `imod` `ieq` `ineq` `ilt` `igt` `ilte` `igte`  
`fadd` `fsub` `fmul` `fdiv` `feq` `flt` `fgt`  
`strConcat` `boolAnd` `boolOr` `boolNot`

---

## Compiler

```
hole run <file.hl>         -- Parse, type-check, interpret
hole check <file.hl>       -- Parse, type-check, report errors/holes
hole check --json <file.hl> -- Machine-readable output for LLM tools
hole repl                  -- Interactive session
```

### LLM Tool Integration

When an LLM calls `hole check --json file.hl`, it receives:

```json
{
  "errors": [
    {
      "message": "3:5: type mismatch: expected Int, got String"
    }
  ],
  "holes": [
    {
      "line": 3,
      "col": 25,
      "expected_type": "Int",
      "context": "magic x :: Int -> Int = ???"
    }
  ]
}
```

The LLM can use the `expected_type` and `context` fields to generate the missing code.

---

## LLM-First Workflow

1. **Write types first** — LLM generates function signatures
2. **Leave holes** — mark incomplete parts with `???`
3. **Check** — compiler reports expected types for each hole
4. **Fill holes** — LLM generates code matching the expected type
5. **Iterate** — repeat until no holes remain

## Turing Completeness

Hole is Turing-complete via:
- Recursive functions (`gcd`, `factorial`)
- Algebraic data types (natural numbers as `Zero | Succ Nat`)
- Lambda calculus encoding (Church numerals)
