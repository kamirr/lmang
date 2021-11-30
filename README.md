# LMA🆖
---
Meaning: *to compute* in Poleese

Pronunciation: *lee-ming* (g as in mingle)

---

## About
LMA🆖 emb**a**ldens you to write safe and efficient software in an ergonomic manner. It does so in a number of ways, most important of them being:

* Dynamic typing, ridding you of nosy compilers.
* Weak typing, so that it works even if the types don’t match exactly.
* Implicit opening bracket in contexts where it’s possible, that being loops, function definitions and such, so that you don’t waste time arguing about a newline preceding it (inspired by [Zimbu](http://www.zimbu.org/)).
* No brackets at all around arguments in call expressions, because it’s tedious to write those. Same goes for any other delimiter and separator that the compiler can reasonably deduce.
* Easy to remember and internalize arithmetics. No more PEMDAS. No need to check precedence rules in the documentation. All binary operators parse right to left - always.

However, the largest advantage of LMA🆖 compared to any other language is its readability. In this regard, it outclasses even Rust. Such feat could only be achieved using two novel approaches:

* All keywords are carefully selected emoji. This makes them easily distinguishable, as you don’t need to parse a sequence of latin letters - or worse - other, even more abstract symbols. Instead, the most primitive part of your brain can instantly process the image on your screen and conclude that before you stands a bald person `🧑‍🦲` (not to confuse with bald man `👨‍🦲` or bald woman `👩‍🦲`). That’s when you know a block is being closed, and can proceed with spending your precious brainpower on tasks far more constructive than staring at this strangely shaped squiggle: `}`.
* The code is indented using the most natural approach - Fibonacci Indentation. Close to the left border of your text-editing widget small differences are easily distinguished, hence even 1-space indent is sufficient. The farther to the right you look, the harder it is to follow the code without the reference of the previously mentioned border. Thusly, the indentation should be more pronounced, which is exactly what Fibonacci Indentation achieves. Moreover, the relative lengths of indentation levels will approach the golden ratio, which makes the bright code on your screen much easier on your eyes.

## Intro
Examples are the best way to start coding in a new language, especially one as simple as LMA🆖. Consider the following program that writes “Hello World” to the standard output and exits:
```
📞🗣🧵Hello World🧵
```
Let’s break it down.

`📞` evaluates expressions to its right, assumes that the first one is a function and then calls it, forwarding all other values to the callee.

`🗣` is a built-in function that takes any number of arguments and prints  them in a single line.

`🧵…🧵` creates a string variable containing the characters between spools.

That was easy, huh? Let’s throw some flow control in there and compute a couple elements of the Fibonacci Sequence:

```
📦
 👶 fib = 🧰 n ➡️ 
  ❓n < 2
    1
  🧑‍🦲😡
    👶 🅰️ = 📞fib📦n - 1🧑‍🦲 💪
    👶 🅱️ = 📞fib📦n - 2🧑‍🦲 💪

    🅰️+🅱️
  🧑‍🦲
 🧑‍🦲💪

 📞🗣 📞fib 0  💪
 📞🗣 📞fib 1  💪
 📞🗣 📞fib 5  💪
 📞🗣 📞fib 30 💪
🧑‍🦲
```

I’m sure you already figured out what’s happening, so I’ll be quick:

First of all, we have a couple of distinct expressions, so the program needs to be put in a block. This block must be explicit and is opened by `📦` at the beginning. The closing token is `🧑‍🦲` at the end.

Inside of it you will notice multiple `💪`, those separate expressions in a block. Also, the value of the last expression in the block becomes the entire block’s value. So, what expressions are there?

```
👶 fib = 🧰 n ➡️ 
 ❓n < 2
  1
 🧑‍🦲😡
  👶 🅰️ = 📞fib📦n - 1🧑‍🦲 💪
  👶 🅱️ = 📞fib📦n - 2🧑‍🦲 💪

  🅰️+🅱️
 🧑‍🦲
🧑‍🦲
```

That too is a new concept, `👶 X=Y` creates a new, baby-variable named `X` and with the value obtained by evaluating the `Y` expression. The token starting said expression is `🧰`, a toolbox. That’s no coincidence, it is a function, a new tool of ours! This tool takes a single argument - `n` - and maps (`➡️`) it to a new value.

What might this value be, you wonder. Do not fret — the function wonders as well, hence the `❓`-expression that executes two different blocks depending on the logical state of its condition (`n < 2`). If it is fulfilled, the block that trivially evaluates to `1` is executed. If it’s not, then the unhappy path (`😡`) takes place and we have some more work to do:

```
👶 🅰️ = 📞fib📦n - 1🧑‍🦲 💪
👶 🅱️ = 📞fib📦n - 2🧑‍🦲 💪

🅰️+🅱️
```


But you already know all of this - we first create two new variables (`🅰️` and `🅱️`) with values of the `fib` function evaluated at `n-1` and `n-2`. The `n-X` sub-expressions are wrapped in blocks, as otherwise the expression would be evaluated as `📦📞fib n🧑‍🦲 - 1`. Then the block evaluates to the last expression, that being the sum `🅰️+🅱️`.

The rest is trivial. We have a couple expressions of the form:
```
📞🗣 📞fib N 💪
```

But this is just a nested function call - we call `🗣` passing to it the result of calling fib with a single argument - `N`.

As you must have noticed, we have more tokens that close blocks (`🧑‍🦲`) than we do those that open them (`📦`). As was explained in the *about* section, opening a block is often implied. In this program it happened for `❓`, `😡` and `🧰`. Other expressions that make use of this feature are `😠` (angry-but-not-very-much `❓` path, sometimes referred to as `elif`) and `💔` (premature exit from a block, similar to `break` in other languages).

If you'd like to see some more samples of LMA🆖, take a look at the `examples/` folder. There are no comments of course, as the language is way too readable for them to be useful. To run them, use `carg run --release --bin lmang-exec -- ./examples/….🆖`

## Book
TODO