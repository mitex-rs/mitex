
#import "pageless.typ": *

// Take a look at the file `template.typ` in the file panel
// to customize this template and discover how it works.
#show: project.with(
  title: "MiTeX Draft",
  authors: (
    "OrangeX4",
    "Myriad Dreamin",
  ),
)

// #let pmat = math.mat.with(delim: "(")

// #let tt(..args) = repr(args)

// $tt(1, 2; 3, 4)$

#let mitex = [*MiTeX*]

= APIs of #mitex 

== Compile Specification

```typst
#let command-fn(num, dir: "right"， has-optional: false) = (
  assosication-dir: dir,
  shape: (
    "fixed-len",
    num,
  ),
)
#let symbol(alias: none) = (
  alias: alias,
)
#let compile-spec(spec: none, inherit: true) = (..);
#let spec = mitex.compile-spec((
  // regular command
  hat: command-fn(1),
  // left associated
  limits: command-fn(1, dir: "left"),
  // symbol
  prod: symbol("product"),
  // environment
  pmatrix: env(0,
    ctx: (mitex.matrix-like-ctx, ), 
    handle: math.mat.with(delim: "("),
  )
));
spec: #repr(spec)
```

It produces:

#align(center)[
  spec: (opaque: bytes(4712), extra-scope: (pmatrix: #repr(() => ())))
]

== Convert LaTeX equation to Typst equation

=== Into String

```typst
#let mitex-convert(input, spec: none) = (..);
#repr(mitex-convert("\alpha x"))
```

It produces:

#align(center)[
  #repr("alpha x ")
]

=== Into Content

```typst
#let mitex(input, spec: none) = (..);
#mitex("\alpha x")
```

It produces:

#align(center)[
  $ alpha x $
]

= Specification of LaTeX Commands (for developers)

// todo: translate Rust code descriptions to natural language

Define how a user specifies commands in Typst for #mitex. Mitex is aware of a special case of command, environment, which is in shape of `\begin{e}...\end{e}`.

The specification is structured as following in Rust:

```rs
/// An item of command specification. It is either a normal _command_ or an
/// _environment_.
/// See [Command Syntax] for concept of _command_.
/// See [Environment Syntax] for concept of _environment_.
///
/// [Command Syntax]: https://latexref.xyz/LaTeX-command-syntax.html
/// [Environment Syntax]: https://latexref.xyz/Environment-syntax.html
pub enum CommandSpecItem {
    Cmd(CmdShape),
    Env(EnvShape),
}

/// Command specification that contains a set of commands and environments.
pub struct CommandSpecRepr {
    /// A map from command name to command specification
    pub commands: fxhash::FxHashMap<String, CommandSpecItem>,
}
```

The specification will be passed to #mitex for converting LaTeX code correctly. For example, #mitex Parser uses it to produce an AST that respect the shape of commands.

Note: since we need to process environments statically, users cannot override the `\begin` and `\end` commands.

```rust
pub struct CmdShape {
    /// Describes how we could match the arguments of a command item.
    pub args: ArgShape,
    /// Makes the command alias to some Typst handler.
    /// For exmaple, alias `\prod` to Typst's `product`
    pub alias: Option<String>,
}

pub struct EnvShape {
    /// Describes how we could match the arguments of an environment item.
    pub args: ArgPattern,
    /// Specifies how we could process items before passing them
    /// to the Typst handler
    pub ctx_feature: ContextFeature,
    /// Makes the command alias to some Typst handler.
    /// For exmaple, alias `\pmatrix` to a Typst function `pmat` in scope.
    pub alias: Option<String>,
}

/// An efficient pattern used for argument matching.
///
/// There are four kinds of pattern. The most powerful one is
/// [`ArgPattern::Glob`], which matches an sequence of input as arguments. Among
/// these four kinds, [`ArgPattern::Glob`] can already match all possible inputs
/// in our use cases. But one should specify a fixed length pattern
/// ([`ArgPattern::FixedLenTerm`]), a range length pattern
/// ([`ArgPattern::RangeLenTerm`]), or a greedy pattern
/// ([`ArgPattern::Greedy`]) to achieve better performance.
///
/// Let us look at usage of a glob pattern by \sqrt, which is `{,b}t`.
///
/// - Example 1. For `\sqrt{2}{3}`, parser requires the pattern to match with an
///   encoded string `tt`. Here, `{,b}t` matches and yields the string `t`
///   (which corresponds to `{2}`).
///
/// - Example 2. For `\sqrt[1]{2}{2}`, parser requires the pattern to match with
///   an encoded string `btt`. Here, `{,b}t` matches and yields the string `bt`
///   (which corresponds to `[1]{2}`).
///
/// Kinds of item to match:
/// - Bracket/b: []
/// - Parenthesis/p: ()
/// - Term/t: any remaining terms, typically {} or a single char
///
/// Note: any prefix of the argument pattern are matched during the parse stage,
/// so you need to check whether it is complete in later stages.
pub enum ArgPattern {
    /// No arguments are passed, i.e. this is processed as a variable in Typst.
    ///
    /// E.g. `\alpha` => `$alpha$`, where `\alpha` has an argument pattern of
    /// `None`
    None,
    /// Fixed length pattern, equivalent to repeat `{,t}` for `x` times
    ///
    /// E.g. `\hat x y` => `$hat(x) y$`, where `\hat` has an argument pattern of
    /// `FixedLenTerm(1)`
    ///
    /// E.g. `1 \sum\limits` => `$1 limits(sum)$`, where `\limits` has an
    /// argument pattern of `FixedLenTerm(1)`
    FixedLenTerm(u8),
    /// Range length pattern (matches as much as possible), equivalent to
    /// repeat `t` for `x` times, then repeat `{,t}` for `y` times.
    ///
    /// No example
    RangeLenTerm(u8, u8),
    /// Receives any items as much as possible, equivalent to `*`.
    ///
    /// E.g. \over, \displaystyle
    Greedy,
    /// The most powerful pattern, but slightly slow.
    /// Note that the glob must accept the whole prefix of the input.
    ///
    /// E.g. \sqrt has a glob argument pattern of `{,b}t`
    ///
    /// Description of the glob pattern:
    /// - {,b}: first, it matches a bracket option, e.g. `\sqrt[3]`
    /// - t: it then matches a single term, e.g. `\sqrt[3]{a}` or `\sqrt{a}`
    Glob(Arc<str>),
}

/// Shape of arguments
/// With direction to match since
/// Note: We currently only support
/// - `Direction::Right` with any `ArgPattern`
/// - `Direction::Left` with `ArgPattern::FixedLenTerm(1)`
/// - `Direction::Infix` with `ArgPattern::Greedy`
pub enum ArgShape {
    /// A command that associates with the right side of items.
    ///
    /// E.g. `\hat`
    Right(ArgPattern),
    /// A command that associates with the left side of items, and with
    /// `ArgPattern::FixedLenTerm(1)`.
    ///
    /// E.g. `\limits`
    Left1,
    /// A command that associates with both side of items, and with
    /// `ArgPattern::Greedy`, also known as infix operators.
    ///
    /// E.g. `\over`
    InfixGreedy,
}

pub enum ContextFeature {
    /// No special feature
    None,
    /// Parse content like mat arguments
    IsMatrix,
    /// Parse content like cases
    IsCases,
}
```

Sample ASTs:

#let cc0 = counter("sample")
#let sample-counter() = {
  cc0.step()
  cc0.display()
}

Sample #sample-counter():

```tex
\frac{ a }{ b }␠
```

```coffee
Command(
  CommandName("frac"),
  Argument(Curly(Space(" "), Word("a"), Space(" "))),
  Argument(Curly(Space(" "), Word("b"), Space(" "))),
),
```

Sample #sample-counter():

Special direction of association are processed:

```tex
\sum\limits _x f(x)␠
```

```coffee
AttachItem(
  Item(Command(
    CommandName("limits"),
    Argument(Command(CommandName("sum")))
  )),
  Bottom(Word("x"))
),
Word("f"),
# used for paren escaping (a special case in typst)
Paren(Word("x")),
```

Sample #sample-counter():

Spaces are not omitted. This is useful for processing `\text`.

```tex
\frac 1  2 ␠
```

```coffee
Command(
  CommandName("frac"),
  Argument(Word("1")),
  Argument(Word("2")),
),
Space(" "),
```

Sample #sample-counter():

Attached items are identified.

```tex
x_1''^2␠
```

```coffee
AttachItem(
  Item(Word("x")),
  Superscript(Word("'"), Word("'"), Word("2")),
  Subscript(1),
),
```

Sample #sample-counter():

Attached items are identified (case 2).

```tex
x''_1␠
```

```coffee
AttachItem(
  Item(Word("x")),
  Superscript(Word("'"), Word("'")),
  Subscript(1),
),
```

Sample #sample-counter():

Apostrophes without an attached target will become regular text.

```tex
''␠
```

```coffee
Word("'")
Word("'")
```

Sample #sample-counter():

Apostrophes that in the position of a command argument will become regular text.

```tex
\frac''␠
```

```coffee
Command(
  CommandName("frac"),
  Argument(Word("'")),
  Argument(Word("'")),
),
```
