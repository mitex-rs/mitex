
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

= Specification to LaTeX Commands (for developers)

// todo: translate rust code descriptions to natural language

Define how one user specifies commands in typst for #mitex. And it is aware of a special case of command, environment, which is in shape of `\begin{e}...\end{e}`.

The specification is structured as following in rust:

```rs
pub enum Command {
    Cmd(CmdShape),
    Env(EnvShape),
}

pub struct CommandSpec {
    pub commands: HashMap<String, Command>,
}
```

The specification will be passed to #mitex Parser, and used for produce ASTs which respects shape of commands.

Note: since we need process environments statically, users cannot override `\begin` and `\end` command.

```rust
pub struct CmdShape {
    /// Describing how could we matches the arguments of a command item
    pub args: ArgShape,
    /// Alias command for typst handler
    /// For exmaple, alias `\prod` to typst's `product`
    pub alias: Option<String>,
}

pub struct EnvShape {
    /// Describing how could we matches the arguments of an environment item
    pub args: ArgPattern,
    /// Specify how could we process items before passing them
    /// to the typst handler
    pub ctx_feature: ContextFeature,
    /// Alias command for typst handler
    /// For exmaple, alias `pmatrix` to `pmat`
    /// And specify `let pmat = math.mat.with(delim: "(")`
    /// in scope
    pub alias: Option<String>,
}

/// An efficient pattern used for matching.
/// It is essential regex things but one
/// can specify the pattern by fixed, range,
/// or greedy length to achieve higher performance.
///
/// Let us show usage of glob pattern by \sqrt, which is `{,b}t`
/// Exp 1. For `\sqrt{2}{3}`, parser
///   requires the pattern to match with `tt`,
///   Here, `{,b}t` matches and
///   yields string `t` (correspond to `{2}`)
/// Exp 2. For `\sqrt[1]{2}{2}`, parser
///   requires the pattern to match with `btt`,
///   Here, `{,b}t` matches and
///   yields string `bt` (correspond to `[1]{2}`)
///
/// Kind of item to match:
/// - Bracket/b: []
/// - Parenthesis/p: ()
/// - Term/t: any rest of terms, typically {} or single char
pub enum ArgPattern {
    /// None of arguments is passed, i.e. it is processed as a
    /// variable in typst.
    /// Note: this is different from FixedLenTerm(0)
    /// Where, \alpha is None, but not FixedLenTerm(0)
    /// E.g. \alpha => $alpha$
    None,
    /// Fixed length pattern, equivalent to `/t{x}/g`
    /// E.g. \hat x y => $hat(x) y$,
    /// E.g. 1 \sum\limits => $1 limits(sum)$,
    FixedLenTerm(u8),
    /// Range length pattern (as much as possible),
    /// equivalent to `/t{x,y}/g`
    /// No example
    RangeLenTerm(u8, u8),
    /// Receive terms as much as possible,
    /// equivalent to `/t*/g`
    /// E.g. \over, \displaystyle
    Greedy,
    /// Most powerful pattern, but slightly slow
    /// Note that the glob must accept all prefix of the input
    ///
    /// E.g. \sqrt has a glob pattern of `{,b}t`
    /// Description:
    /// - {,b}: first, it matches an bracket option, e.g. `\sqrt[3]`
    /// - t: it later matches a single term, e.g. `\sqrt[3]{a}` or `\sqrt{a}`
    /// Note: any prefix of the glob is valid in parse stage hence you need to
    /// check whether it is complete in later stage.
    Glob(Arc<str>),
}

// struct ArgShape(ArgPattern, Direction);

/// Shape of arguments
/// With direction to match since
/// Note: We currently only support
/// - `Direction::Right` with any `ArgPattern`
/// - `Direction::Left` with `ArgPattern::FixedLenTerm(1)`
/// - `Direction::Infix` with `ArgPattern::Greedy`
pub enum ArgShape {
    /// A command that assosicates with right side of items.
    /// E.g. \hat
    Right(ArgPattern),
    /// A command that assosicates with left side of items, and with `ArgPattern::FixedLenTerm(1)`.
    /// E.g. \limits
    Left1,
    /// A command that assosicates with both side of items, and with `ArgPattern::Greedy`.
    /// Also known as infix operators.
    /// E.g. \over
    InfixGreedy,
}

pub enum ContextFeature {
    // 需要使用&和//分隔上下文中的内容
    IsMatrix,
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
# used for paren escaping (special case in typst)
Paren(Word("x")),
```

Sample #sample-counter():

Spaces are not omitted. It is useful for processing `\text`.

```tex
\frac 1  2 ␠
```

```coffee
Command(
  CommandName("frac"),
  Space(" "),
  Argument(Word("1")),
  Space("  ")
  Argument(Word("2")),
),
Space(" "),
```

Sample #sample-counter():

Attach items are identified.

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

Attach items are identified (case 2).

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

Apostrophes without attach target will become regular text.

```tex
''␠
```

```coffee
Word("'")
Word("'")
```

Sample #sample-counter():

Apostrophes that occupy position of command argument will become regular text.

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
