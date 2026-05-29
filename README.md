# 🕳️ Hole

**An experimental LLM-first functional programming language with Hindley-Milner type inference and a Rust compiler.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Why an LLM-first language?

LLMs are remarkably good at generating code — but mainstream languages weren't designed for them. They were designed for humans: terse syntax, implicit behavior, decades of accumulated cruft. An LLM doesn't need any of that. What it needs is a language that **maximizes feedback density** and **minimizes the space of mistakes**.

### The problem with writing code for humans

When an LLM writes Python or JavaScript, it operates in a high-entropy environment:

- **Implicit coercion** — `"5" + 3` doesn't fail, it silently produces `"53"`. The LLM gets no error signal.
- **Null everywhere** — `x.y` crashes at runtime if `x` happens to be null. No compile-time guard.
- **Loose structure** — the LLM can write `if x = 5` instead of `if x == 5` in a language with assignment-as-expression. TypeScript catches this, Python doesn't.
- **Silent failures** — `try { ... } catch { /* nothing */ }` is valid in every major language. The LLM has no incentive to handle errors.

Every one of these is a footgun the LLM can step on. The compiler says nothing. The code runs. Something is subtly wrong.

### Flipping the relationship

A language designed for **LLMs as the primary author** inverts the design priorities:

| Property | Why it helps LLMs |
|----------|-------------------|
| **No implicit conversions** | `Int + String` is a type error. LLM learns immediately. |
| **No null** | `Option a` forces the LLM to handle both cases. |
| **Pure by default** | Functions can't launch missiles unless wrapped in `IO`. |
| **Required type annotations** | A function signature IS the prompt for its body. |
| **Exhaustive pattern matching** | Forget a variant → compile error. LLM fills it in. |
| **No silent error paths** | Errors are `Result e a`, not exceptions you can ignore. |

The language doesn't trust the LLM to "be careful." It **structurally prevents** whole categories of bugs that LLMs are prone to.

### The `???` workflow

This is the central LLM-first innovation. You write the types first — which serve as the LLM's prompt — and leave `???` holes for the body:

```haskell
-- Step 1: write the type signature (the prompt)
magic :: List Int -> Int

-- Step 2: leave a hole
magic xs = ???
```

The compiler responds:

```
hole: 3:17: hole of type Int
  3 | magic xs = ???

  bindings in scope: xs :: List Int
```

The LLM sees: "I need to produce an `Int` from a `List Int`." It generates `sum xs` or `length xs` or `head xs`. Type-driven code synthesis — the compiler tells the LLM exactly what contract it needs to fulfill.

This is the opposite of the traditional workflow (write code, get type errors, fix). Here, the types are written first as a **specification**, and the LLM fills in the implementation guided by the compiler's feedback.

---

## Language Tour

### Functions

```haskell
-- params before ::, type annotation, =, then body
add a b :: Int -> Int -> Int = a + b

factorial n :: Int -> Int =
  if n <= 1 then 1 else n * factorial (n - 1)

-- zero-param: evaluates at use site
pi :: Float = 3.14159
```

### Algebraic Data Types

```haskell
type Option a = None | Some a

type List a = Nil | Cons a (List a)
```

### Pattern Matching

Exhaustiveness is enforced — the compiler rejects incomplete matches:

```haskell
sum xs :: List Int -> Int =
  match xs with
    Nil       -> 0
    Cons x xs -> x + sum xs
```

### Higher-order Functions

```haskell
map f xs :: (a -> b) -> List a -> List b =
  match xs with
    Nil       -> Nil
    Cons x xs -> Cons (f x) (map f xs)

foldl f acc xs :: (b -> a -> b) -> b -> List a -> b =
  match xs with
    Nil       -> acc
    Cons x xs -> foldl f (f acc x) xs
```

### Lambdas and currying

```haskell
double :: Int -> Int = \x -> x * 2

apply f x :: (Int -> Int) -> Int -> Int = f x
```

### IO is explicit

```haskell
main :: IO () = do
  println "Enter name:"
  name <- readLine
  println ("Hello, " ++ name)
```

### Holes

```haskell
-- compiler: "hole of type Int"
magic x :: Int -> Int = ???
```

---

## Compiler Pipeline

```
source.hl → lexer → parser → type checker (HM inference) → interpreter
                                        ↓
                                  hole detection
                                  exhaustiveness check
```

Written in Rust (~3,500 lines). Tree-walking interpreter — fast enough for development, type-checks in milliseconds.

### Modes

```bash
hole run file.hl          # compile & run
hole check file.hl        # type-check, report errors & holes
hole check --json file.hl # structured output for LLM tools
hole repl                 # interactive session
```

### JSON output (for LLM integration)

```json
{
  "errors": [],
  "holes": [{
    "line": 3,
    "col": 25,
    "expected_type": "Int",
    "context": "magic x :: Int -> Int = ???"
  }]
}
```

---

## Examples

| Program | What it demonstrates | Output |
|---------|---------------------|--------|
| `hello.hl` | Functions, IO | `42` |
| `gcd.hl` | Recursion, conditionals | `6 21` |
| `fib.hl` | Recursion depth | `55 6765` |
| `list.hl` | ADTs, pattern matching | `3 6` |
| `hof.hl` | map, filter, foldl | `15 30 12` |
| `peano.hl` | Peano arithmetic | `7 12` |
| `sort.hl` | Quicksort, let-in | `28` |
| `adt.hl` | Trees, multiple ADTs | `6 42 0` |
| `functions.hl` | Lambdas, currying, higher-order | `120 42 6 7` |
| `holes.hl` | Hole detection | _(reports holes)_ |

Run any example: `hole run examples/sort.hl`

---

## Installation

```bash
git clone https://github.com/sjusjdkzhsiqjqbz-rgb/hole-lang.git
cd hole-lang
cargo build --release
./target/release/hole run examples/hello.hl
```

Requires Rust 1.85+.

---

## Status

**Experimental.** This is a research project exploring what happens when you design a language for LLMs rather than humans. It is:

- ✅ **Turing complete** (recursive functions, ADTs)
- ✅ **Hindley-Milner type inference** with let-polymorphism
- ✅ **Exhaustive pattern matching**
- ✅ **Hole-driven development** with `???`
- ✅ **IO type** for explicit effects
- ✅ **JSON output** for LLM tool use
- ⬜ Module imports
- ⬜ Type classes / traits
- ⬜ Code generation (currently interpreted)

---

## License

MIT
