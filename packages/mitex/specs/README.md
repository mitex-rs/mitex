# MiTeX Command Specification

This includes ways to define specs, which can be used to define everything in the existing standard packages [latex-std](./latex/standard.typ).

Even if you don't know Rust at all, you can still add missing TeX commands to MiTeX by modifing [latex-std](./latex/standard.typ), since they are written in typst! You can open an issue to acquire the commands you want to add, or you can edit the files and submit a pull request.


## Concepts

Currently, MiTeX maintains following three parts of code:

- A TeX parser library written in **Rust**, see [mitex-lexer](./crates/mitex-lexer) and [mitex-parser](./crates/mitex-parser).
- A TeX to Typst converter library written in **Rust**, see [mitex](./crates/mitex).
- A list of TeX packages and comamnds written in **Typst**, which then used by the typst package, see [MiTeX Command Specification](./packages/mitex/specs).

For a translation process, for example, we have:

```
\frac{1}{2}

===[parser]===> AST ===[converter]===>

#eval("$frac(1, 2)$", scope: (frac: (num, den) => $(num)/(den)$))
```

You can use the `#mitex-convert()` function to get the Typst Code generated from LaTeX Code.

To achieve this, we need to define four components for LaTeX commands:

- `cmd`: The name of the LaTeX command. Since LaTeX commands all start with `\`, we remove the leading `\` and use the command name as the key in the dictionary.
  - There is also the concept of environments, for example, `\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}` is a matrix environment.
- `args`: The argument pattern of the LaTeX command. For example:
  - `\alpha` has no arguments;
  - `\hat{x}` matches one argument on the right;
  - `\frac{1}{2}` matches two arguments on the right;
  - `\sqrt[3]{2}` includes optional arguments;
  - `\sum\limits` has `limits` matching one argument on the left;
  - `\displaystyle` greedily matches all arguments on the right;
  - `x \over y` as an infix operator greedily matches all arguments on both sides.
- `alias`: The alias of the LaTeX command in Typst.
  - The alias can be an existing symbol or function, for example, `alpha` and `binom(n, k)`;
  - It can also be a key in the `mitex-scope` for `eval`, for example, our self-defined `frac`.
- `handle`: The value in the `mitex-scope`, which is our self-defined symbol or processing function.
  - For example, the `frac` key corresponds to the value `(num, den) => $(num)/(den)$`.

The parser only needs `cmd` and `args` to generate an AST, while the converter requires `cmd`, `args`, and `alias` to generate the corresponding Typst code. Since both the parser and converter are part of the WASM plugin written in Rust, we refer to the dictionary composed of `cmd`, `args`, and `alias` as spec and pre-compile it into the wasm.

At the Typst level, we need `alias` and `handle`, which are then combined into the `mitex-scope` passed to the `eval` function as the `scope` parameter.

Therefore, we need to define the spec here, and later we will also support custom spec.

## Reference

### `define-sym`

Define a normal symbol, as no-argument commands like `\alpha`.

```typst
#let define-sym(s, sym: none) = { .. }
```

**Arguments:**
- s (str): Alias command for typst handler.
  For example, alias `\prod` to typst's `product`.
- sym (content): The specific content, as the value of alias in mitex-scope.
  For example, there is no direct alias for \negthinspace symbol in typst,
  but we can add `h(-(3/18) * 1em)` ourselves

**Return:** A spec item and a scope item (none for no scope item)


### `define-greedy-cmd`

Define a greedy command, like `\displaystyle`.

```typst
#let define-greedy-cmd(s, handle: none) = { .. }
```

**Arguments:**
- s (str): Alias command for typst handler.
  For example, alias `\displaystyle` to typst's `mitexdisplay`, as the key in mitex-scope.
- handle (function): The handler function, as the value of alias in mitex-scope.
  It receives a content argument as all greedy matches to the content
  For example, we define `mitexdisplay` to `math.display`

**Return:** A spec item and a scope item (none for no scope item)


### `define-infix-cmd`

Define an infix command, like `\over`.

```typst
#let define-infix-cmd(s, handle: none) = { .. }
```

**Arguments:**
- s (str): Alias command for typst handler.
  For example, alias `\over` to typst's `frac`, as the key in mitex-scope.
- handle (function): The handler function, as the value of alias in mitex-scope.
  It receives two content arguments, as (prev, after) arguments.
  For example, we define `\over` to `frac: (num, den) => $(num)/(den)$`

**Return:** A spec item and a scope item (none for no scope item)


### `define-glob-cmd`

Define a glob (Global Wildcard) match command with a specified pattern for matching args
Kind of item to match:

- Bracket/b: []
- Parenthesis/p: ()
- Term/t: any rest of terms, typically {} or single char

```typst
#let define-glob-cmd(pat, s, handle: none) = { .. }
```

**Arguments:**
- pat (pattern): The pattern for glob-cmd
  For example, `{,b}t` for `\sqrt` to support `\sqrt{2}` and `\sqrt[3]{2}`
- s (str): Alias command for typst handler.
  For example, alias `\sqrt` to typst's `mitexsqrt`, as the key in mitex-scope.
- handle (function): The handler function, as the value of alias in mitex-scope.
  It receives variable length arguments, for example `(2,)` or `([3], 2)` for sqrt.
  Therefore you need to use `(.. arg) = > {..}` to receive them.

**Return:** A spec item and a scope item (none for no scope item)


### `define-cmd`

Define a command with a fixed number of arguments, like `\hat{x}` and `\frac{1}{2}`.

```typst
#let define-cmd(num, alias: none, handle: none) = { .. }
```

**Arguments:**
- num (int): The number of arguments for the command.
- alias (str): Alias command for typst handler.
  For example, alias `\frac` to typst's `frac`, as the key in mitex-scope.
- handle (function): The handler function, as the value of alias in mitex-scope.
  It receives fixed number of arguments, for example `frac(1, 2)` for `\frac{1}{2}`.

**Return:** A spec item and a scope item (none for no scope item)


### `define-env`

Define an environment with a fixed number of arguments, like `\begin{alignedat}{2}`.

```typst
#let define-env(num, alias: none, handle: none) = { .. }
```

**Arguments:**
- num (int): The number of arguments as environment options for the environment.
- alias (str): Alias command for typst handler.
  For example, alias `\begin{alignedat}{2}` to typst's `alignedat`,
  and alias `\begin{aligned}` to typst's `aligned`, as the key in mitex-scope.
- handle (function): The handler function, as the value of alias in mitex-scope.
  It receives fixed number of named arguments as environment options,
  for example `alignedat(arg0: ..)` or `alignedat(arg0: .., arg1: ..)`.
  And it receives variable length arguments as environment body,
  Therefore you need to use `(.. arg) = > {..}` to receive them.

**Return:** A spec item and a scope item (none for no scope item)


### `define-cases-env`

Define a cases environment.

```typst
#let define-cases-env(alias: none, handle: none) = { .. }
```

**Arguments:**
- alias (str): Alias command for typst handler.
  For example, alias `\begin{rcases}` to typst's `rcases`,
- handle (function): The handler function, as the value of alias in mitex-scope.
  For example, define `math.cases.with(reverse: true)` for `rcases` in mitex-scope.

**Return:** A spec item and a scope item (none for no scope item)


### `define-matrix-env`

Define an matrix environment with a fixed number of arguments, like \begin{pmatrix}

```typst
#let define-matrix-env(num, alias: none, handle: none) = { .. }
```

**Arguments:**
- num (int): The number of arguments as environment options for the environment.
- alias (str): Alias command for typst handler.
  For example, alias `\begin{pmatrix}` to typst's `pmatrix`, as the key in mitex-scope.
- handle (function): The handler function, as the value of alias in mitex-scope.
  It receives fixed number of named arguments as environment options,
  for example `pmatrix(arg0: ..)` or `pmatrix(arg0: .., arg1: ..)`.
  And it receives variable length arguments as environment body,
  for matrix environment, it just like the arguments for `mat(1,2; 3, 4)` in equation mode,
  That is, to receive a two-dimensional array,
  like `pmatrtix((1, 2,), (3, 4,))` in script mode.
  Therefore you need to use `(.. arg) = > {..}` to receive them.

**Return:** A spec item and a scope item (none for no scope item)


### `sym`

Define a symbol without alias and without handler function, like \alpha => alpha

```typst
#let sym = ((kind: "sym"), none)
```

**Return:** A spec item and no scope item (none for no scope item)


### `of-sym`

```typst
#let of-sym(handle) = ((kind: "sym"), (handle: handle))
```

Define a symbol without alias and with handler function,
like \negthinspace => h(-(3/18) * 1em)

**Arguments:**

- handle (function): The handler function, as the value of alias in mitex-scope.
  For example, define `negthinspace` to handle `h(-(3/18) * 1em)` in mitex-scope

**Return:** A symbol spec and a scope item


### `left1-op`

Define a left1-op command without handler, like `\limits` for `\sum\limits`

```typst
#let left1-op(alias) = ((kind: "cmd", args: ( kind: "left1" ), alias: alias), none)
```

**Arguments:**

- alias (str): Alias command for typst handler.
  For example, alias `\limits` to typst's `limits`
  and alias `\nolimits` to typst's `scripts`

**Return:** A cmd spec and no scope item (none for no scope item)


### `cmd1`

Define a cmd1 command like \hat{x} => hat(x)

```typst
#let cmd1 = ((kind: "cmd1"), none)
```

**Return:** A cmd1 spec and a scope item (none for no scope item)


### `cmd2`

Define a cmd2 command like \binom{1}{2} => binom(1, 2)

```typst
#let cmd2 = ((kind: "cmd2"), none)
```

**Return:** A cmd2 spec and a scope item (none for no scope item)


### `matrix-env`

Define a matrix environment without handler

```typst
#let matrix-env = ((kind: "matrix-env"), none)
```

**Return:** A matrix-env spec and a scope item (none for no scope item)


### `normal-env`

Define a normal environment with handler

```typst
#let normal-env(handle) = ((kind: "normal-env"), (handle: handle))
```

**Arguments:**

- handle (function): The handler function, as the value of alias in mitex-scope.
  For example, define how to handle `aligned(..)` in mitex-scope

**Return:** A normal-env spec and a scope item (none for no scope item)


### `process-spec`

Receives a dictionary of definitions composed of the above functions, and processes them to return a dictionary containing spec and scope.

```typst
#let process-spec(definitions) = { .. }
```