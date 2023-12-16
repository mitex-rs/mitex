use phf::phf_map;
use wasm_minimal_protocol::*;
use regex::bytes::Regex;
use once_cell::sync::Lazy;

initiate_protocol!();

enum MapType {
  Skip,  // skip space and newline
  Normal,  // normal map, add a space after the replacement
  NoSpace,  // normal map, but no space after the replacement
  SqrtN,  // sqrt with n
  MatrixBegin,  // matrix begin, for convert & to , and \\ to ;
  MatrixEnd,  // matrix end, for convert & to & and \\ to \
  Text,  // text mode, like \text and \operatorname
  Command,  // commands like "\alpha", add a space after the command
  Default,  // straight copy, add a space after the string
}

// a global static map list, from regex to string
static TOKEN_MAP_LIST: Lazy<Vec<(Regex, &[u8], MapType)>> = Lazy::new(|| { vec![
  // Spaces and newline
  (Regex::new(r"^[ \n]+").unwrap(), b"", MapType::Skip),
  // Escape symbols
  (Regex::new(r"^,").unwrap(), b"\\,", MapType::Normal),
  (Regex::new(r"^;").unwrap(), b"\\;", MapType::Normal),
  (Regex::new(r"^/").unwrap(), b"\\/", MapType::Normal),
  (Regex::new(r"^\\\|").unwrap(), b"||", MapType::Normal),
  (Regex::new(r"^\\\&").unwrap(), b"amp", MapType::Normal),
  (Regex::new(r"^\\\#").unwrap(), b"hash", MapType::Normal),
  (Regex::new(r"^\\\%").unwrap(), b"percent", MapType::Normal),
  (Regex::new(r"^\\\$").unwrap(), b"dollar", MapType::Normal),
  // Hack for {\displaystyle ...} and {\rm ...}
  (Regex::new(r"^\{[ \n]*\\displaystyle[ \n]").unwrap(), b"display(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\textstyle[ \n]").unwrap(), b"inline(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\scriptstyle[ \n]").unwrap(), b"script(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\scriptscriptstyle[ \n]").unwrap(), b"sscript(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\bf[ \n]").unwrap(), b"bold(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\rm[ \n]").unwrap(), b"upright(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\it[ \n]").unwrap(), b"italic(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\sf[ \n]").unwrap(), b"sans(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\frak[ \n]").unwrap(), b"frak(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\tt[ \n]").unwrap(), b"mono(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\cal[ \n]").unwrap(), b"cal(", MapType::Normal),
  (Regex::new(r"^\{").unwrap(), b"(", MapType::NoSpace),
  // Just a hack for "}{" in "frac{}{}"
  (Regex::new(r"^\}[ \n]*\{").unwrap(), b", ", MapType::NoSpace),
  (Regex::new(r"^\}").unwrap(), b")", MapType::NoSpace),
  // Left/Right
  (Regex::new(r"^\\left[ \n]*\.").unwrap(), b"lr(", MapType::Normal),
  (Regex::new(r"^\\left").unwrap(), b"lr(", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\.").unwrap(), b")", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\)").unwrap(), b"))", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\]").unwrap(), b"])", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\\\}").unwrap(), b"})", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\|").unwrap(), b"|)", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\\\|}").unwrap(), b"||)", MapType::Normal),
  // Brackets
  (Regex::new(r"^\(").unwrap(), b"(", MapType::NoSpace),
  (Regex::new(r"^\)").unwrap(), b")", MapType::NoSpace),
  (Regex::new(r"^\[").unwrap(), b"[", MapType::NoSpace),
  (Regex::new(r"^\]").unwrap(), b"]", MapType::NoSpace),
  (Regex::new(r"^\\\{").unwrap(), b"{", MapType::NoSpace),
  (Regex::new(r"^\\\}").unwrap(), b"}", MapType::NoSpace),
  // Sups and subs
  (Regex::new(r"^\^").unwrap(), b"^", MapType::NoSpace),
  (Regex::new(r"^\_").unwrap(), b"_", MapType::NoSpace),
  // Limits and scripts
  (Regex::new(r"^\\max[ \n]*\\limits").unwrap(), b"limits(max)", MapType::NoSpace),
  (Regex::new(r"^\\min[ \n]*\\limits").unwrap(), b"limits(min)", MapType::NoSpace),
  (Regex::new(r"^\\argmax[ \n]*\\limits").unwrap(), b"limits(arg max)", MapType::NoSpace),
  (Regex::new(r"^\\argmin[ \n]*\\limits").unwrap(), b"limits(arg min)", MapType::NoSpace),
  (Regex::new(r"^\\sup[ \n]*\\limits").unwrap(), b"limits(sup)", MapType::NoSpace),
  (Regex::new(r"^\\inf[ \n]*\\limits").unwrap(), b"limits(inf)", MapType::NoSpace),
  (Regex::new(r"^\\sum[ \n]*\\limits").unwrap(), b"limits(sum)", MapType::NoSpace),
  (Regex::new(r"^\\prod[ \n]*\\limits").unwrap(), b"limits(prod)", MapType::NoSpace),
  (Regex::new(r"^\\int[ \n]*\\limits").unwrap(), b"limits(int)", MapType::NoSpace),
  (Regex::new(r"^\\max[ \n]*\\nolimits").unwrap(), b"scripts(max)", MapType::NoSpace),
  (Regex::new(r"^\\min[ \n]*\\nolimits").unwrap(), b"scripts(min)", MapType::NoSpace),
  (Regex::new(r"^\\argmax[ \n]*\\nolimits").unwrap(), b"scripts(arg max)", MapType::NoSpace),
  (Regex::new(r"^\\argmin[ \n]*\\nolimits").unwrap(), b"scripts(arg min)", MapType::NoSpace),
  (Regex::new(r"^\\sup[ \n]*\\nolimits").unwrap(), b"scripts(sup)", MapType::NoSpace),
  (Regex::new(r"^\\inf[ \n]*\\nolimits").unwrap(), b"scripts(inf)", MapType::NoSpace),
  (Regex::new(r"^\\sum[ \n]*\\nolimits").unwrap(), b"scripts(sum)", MapType::NoSpace),
  (Regex::new(r"^\\prod[ \n]*\\nolimits").unwrap(), b"scripts(prod)", MapType::NoSpace),
  (Regex::new(r"^\\int[ \n]*\\nolimits").unwrap(), b"scripts(int)", MapType::NoSpace),
  // Sqrt
  (Regex::new(r"^\\sqrt[ \n]*\[[ \n]*([0-9]+)[ \n]*\][ \n]*\{").unwrap(), b"", MapType::SqrtN),
  // Aligned
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*aligned[ \n]*\}").unwrap(), b"", MapType::Normal),
  (Regex::new(r"^\\end[ \n]*\{[ \n]*aligned[ \n]*\}").unwrap(), b"", MapType::Normal),
  // Matrices
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*matrix[ \n]*\}").unwrap(), b"mat(delim: #none,", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*pmatrix[ \n]*\}").unwrap(), b"mat(delim: \"(\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*bmatrix[ \n]*\}").unwrap(), b"mat(delim: \"[\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*Bmatrix[ \n]*\}").unwrap(), b"mat(delim: \"{\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*vmatrix[ \n]*\}").unwrap(), b"mat(delim: \"|\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*Vmatrix[ \n]*\}").unwrap(), b"mat(delim: \"||\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*array[ \n]*\}\{[lcr\| \n]+\}").unwrap(), b"mat(delim: #none,", MapType::MatrixBegin),
  (Regex::new(r"^\\end[ \n]*\{[ \n]*[pbBvV]?matrix[ \n]*\}").unwrap(), b")", MapType::MatrixEnd),
  (Regex::new(r"^\\end[ \n]*\{[ \n]*array[ \n]*\}").unwrap(), b")", MapType::MatrixEnd),
  // Spaces
  (Regex::new(r"^\\!").unwrap(), b"#h(-(1/6)*1em);", MapType::Normal),
  (Regex::new(r"^\\,").unwrap(), b"thin", MapType::Normal),
  (Regex::new(r"^\\>").unwrap(), b"med", MapType::Normal),
  (Regex::new(r"^\\:").unwrap(), b"med", MapType::Normal),
  (Regex::new(r"^\\;").unwrap(), b"thick", MapType::Normal),
  (Regex::new(r"^\\[ \n]").unwrap(), b"thick", MapType::Normal),
  (Regex::new(r"^~").unwrap(), b"thick", MapType::Normal),
  // Text
  (Regex::new(r"^\\text[ \n]*\{([^\}]+)\}").unwrap(), b"", MapType::Text),
  (Regex::new(r"^\\operatorname[ \n]*\{([^\}]+)\}").unwrap(), b"op(", MapType::Text),
  (Regex::new(r"^\\operatorname\*[ \n]*\{([^\}]+)\}").unwrap(), b"op(limits: #false, ", MapType::Text),
  (Regex::new(r"^\\operatornamewithlimits[ \n]*\{([^\}]+)\}").unwrap(), b"op(limits: #false, ", MapType::Text),
  // Commands and default
  (Regex::new(r"^\\([a-zA-Z]+)").unwrap(), b"", MapType::Command),
  (Regex::new(r"^[a-zA-Z+\-*!<>=]").unwrap(), b"", MapType::Default),
  (Regex::new(r"^[0-9]+").unwrap(), b"", MapType::Default),
]});

// Command maps, [key] to (replacement, add_space)
static COMMAND_MAP: phf::Map<&'static [u8], (&'static [u8], bool)> = phf_map! {
  // greek letters
  b"alpha" => (b"alpha", true),
  b"beta" => (b"beta", true),
  b"gamma" => (b"gamma", true),
  b"delta" => (b"delta", true),
  b"epsilon" => (b"epsilon", true),
  b"zeta" => (b"zeta", true),
  b"eta" => (b"eta", true),
  b"theta" => (b"theta", true),
  b"iota" => (b"iota", true),
  b"kappa" => (b"kappa", true),
  b"lambda" => (b"lambda", true),
  b"mu" => (b"mu", true),
  b"nu" => (b"nu", true),
  b"xi" => (b"xi", true),
  b"omicron" => (b"omicron", true),
  b"pi" => (b"pi", true),
  b"rho" => (b"rho", true),
  b"sigma" => (b"sigma", true),
  b"tau" => (b"tau", true),
  b"upsilon" => (b"upsilon", true),
  b"phi" => (b"phi", true),
  b"chi" => (b"chi", true),
  b"psi" => (b"psi", true),
  b"omega" => (b"omega", true),
  b"Alpha" => (b"Alpha", true),
  b"Beta" => (b"Beta", true),
  b"Gamma" => (b"Gamma", true),
  b"Delta" => (b"Delta", true),
  b"Epsilon" => (b"Epsilon", true),
  b"Zeta" => (b"Zeta", true),
  b"Eta" => (b"Eta", true),
  b"Theta" => (b"Theta", true),
  b"Iota" => (b"Iota", true),
  b"Kappa" => (b"Kappa", true),
  b"Lambda" => (b"Lambda", true),
  b"Mu" => (b"Mu", true),
  b"Nu" => (b"Nu", true),
  b"Xi" => (b"Xi", true),
  b"Omicron" => (b"Omicron", true),
  b"Pi" => (b"Pi", true),
  b"Rho" => (b"Rho", true),
  b"Sigma" => (b"Sigma", true),
  b"Tau" => (b"Tau", true),
  b"Upsilon" => (b"Upsilon", true),
  b"Phi" => (b"Phi", true),
  b"Chi" => (b"Chi", true),
  b"Psi" => (b"Psi", true),
  b"Omega" => (b"Omega", true),
  // Functions
  b"sin" => (b"sin", true),
  b"cos" => (b"cos", true),
  b"tan" => (b"tan", true),
  b"cot" => (b"cot", true),
  b"sec" => (b"sec", true),
  b"csc" => (b"csc", true),
  b"arcsin" => (b"arcsin", true),
  b"arccos" => (b"arccos", true),
  b"arctan" => (b"arctan", true),
  b"sinh" => (b"sinh", true),
  b"cosh" => (b"cosh", true),
  b"tanh" => (b"tanh", true),
  b"coth" => (b"coth", true),
  b"ln" => (b"ln", true),
  b"log" => (b"log", true),
  b"lg" => (b"lg", true),
  b"lim" => (b"lim", true),
  b"limsup" => (b"limsup", true),
  b"liminf" => (b"liminf", true),
  b"hom" => (b"hom", true),
  b"det" => (b"det", true),
  b"exp" => (b"exp", true),
  b"deg" => (b"deg", true),
  b"gcd" => (b"gcd", true),
  b"lcm" => (b"lcm", true),
  b"dim" => (b"dim", true),
  b"ker" => (b"ker", true),
  b"arg" => (b"arg", true),
  b"Pr" => (b"Pr", true),
  // Limits
  b"max" => (b"max", true),
  b"min" => (b"min", true),
  b"argmax" => (b"op(limits: #true, arg max)", true),
  b"argmin" => (b"op(limits: #true, arg min)", true),
  b"sup" => (b"sup", true),
  b"inf" => (b"inf", true),
  b"sum" => (b"sum", true),
  b"prod" => (b"product", true),
  // Integrals
  b"int" => (b"integral", true),
  b"iint" => (b"integral.double", true),
  b"iiint" => (b"integral.triple", true),
  b"oint" => (b"integral.cont", true),
  b"oiint" => (b"integral.surf", true),
  b"oiiint" => (b"integral.vol", true),
  // Operators
  b"mod" => (b"mod", true),
  b"cdot" => (b"dot.c", true),
  b"times" => (b"times", true),
  b"oplus" => (b"plus.circle", true),
  b"ominus" => (b"minus.circle", true),
  b"pm" => (b"plus.minus", true),
  b"mp" => (b"minus.plus", true),
  b"div" => (b"div", true),
  b"star" => (b"star", true),
  b"cap" => (b"sect", true),
  b"cup" => (b"union", true),
  b"in" => (b"in", true),
  b"notin" => (b"in.not", true),
  b"subset" => (b"subset", true),
  b"subseteq" => (b"subset.eq", true),
  b"neq" => (b"!=", true),
  b"lt" => (b"<", true),
  b"gt" => (b">", true),
  b"le" => (b"<=", true),
  b"ge" => (b">=", true),
  b"leq" => (b"<=", true),
  b"geq" => (b">=", true),
  b"leqslant" => (b"lt.eq.slant", true),
  b"geqslant" => (b"gt.eq.slant", true),
  b"approx" => (b"approx", true),
  // Hacks
  b"over" => (b")/(", false),
  // Accents
  b"not" => (b"cancel", false),
  b"grave" => (b"grave", false),
  b"acute" => (b"acute", false),
  b"hat" => (b"hat", false),
  b"tilde" => (b"tilde", false),
  b"bar" => (b"macron", false),
  b"breve" => (b"breve", false),
  b"dot" => (b"dot", false),
  b"ddot" => (b"dot.double", false),
  b"dddot" => (b"dot.triple", false),
  b"ddddot" => (b"dot.quad", false),
  b"H" => (b"acute.double", false),
  b"v" => (b"caron", false),
  b"vec" => (b"arrow", false),
  b"overrightarrow" => (b"arrow", false),
  b"overleftarrow" => (b"arrow.l", false),
  b"overline" => (b"overline", false),
  b"underline" => (b"underline", false),
  // Styles and variants
  b"bold" => (b"bold", false),
  b"mathbf" => (b"bold", false),
  b"boldsymbol" => (b"bold", false),
  b"mathrm" => (b"upright", false),
  b"mathit" => (b"italic", false),
  b"mathsf" => (b"sans", false),
  b"mathfrak" => (b"frak", false),
  b"mathtt" => (b"mono", false),
  b"mathbb" => (b"bb", false),
  b"mathcal" => (b"cal", false),
  // Functions with no space
  b"frac" => (b"frac", false),
  b"cfrac" => (b"cfrac", false),
  b"dfrac" => (b"dfrac", false),
  b"tfrac" => (b"tfrac", false),
  b"binom" => (b"binom", false),
  // Symbols
  b"infty" => (b"oo", true),
  b"leftarrow" => (b"<-", true),
  b"rightarrow" => (b"->", true),
  b"leftrightarrow" => (b"<->", true),
  b"Leftarrow" => (b"arrow.l.double", true),
  b"Rightarrow" => (b"=>", true),
  b"Leftrightarrow" => (b"<=>", true),
  b"larr" => (b"<-", true),
  b"rarr" => (b"->", true),
  b"lrarr" => (b"<->", true),
  b"lArr" => (b"arrow.l.double", true),
  b"rArr" => (b"=>", true),
  b"lrArr" => (b"<=>", true),
  b"Larr" => (b"arrow.l.double", true),
  b"Rarr" => (b"=>", true),
  b"Lrarr" => (b"<=>", true),
  b"longleftarrow" => (b"<--", true),
  b"longrightarrow" => (b"-->", true),
  b"longleftrightarrow" => (b"<-->", true),
  b"Longleftarrow" => (b"<==", true),
  b"Longrightarrow" => (b"==>", true),
  b"Longleftrightarrow" => (b"<==>", true),
  b"to" => (b"->", true),
  b"mapsto" => (b"|->", true),
  b"quad" => (b"space.quad", true),
  b"overbrace{}" => (b"brace.t", true),
  b"underbrace{}" => (b"brace.b", true),
  b"lbrack" => (b"bracket.l", true),
  b"rbrack" => (b"bracket.r", true),
  b"angle" => (b"angle", true),
  b"langle" => (b"angle.l", true),
  b"rangle" => (b"angle.r", true),
  b"measuredangle" => (b"angle.arc", true),
  b"sphericalangle" => (b"angle.spheric", true),
  b"ast" => (b"ast", true),
  b"circledast" => (b"ast.circle", true),
  b"backslash" => (b"backslash", true),
  b"dagger" => (b"dagger", true),
  b"ddagger" => (b"dagger.double", true),
  b"circleddash" => (b"dash.circle", true),
  b"odot" => (b"dot.circle", true),
  b"bigodot" => (b"dot.circle.big", true),
  b"boxdot" => (b"dot.square", true),
  b"cdots" => (b"dots.h.c", true),
  b"ldots" => (b"dots.h", true),
  b"vdots" => (b"dots.v", true),
  b"ddots" => (b"dots.down", true),
  b"sim" => (b"tilde", true),
  b"simeq" => (b"tilde.eq", true),
  b"backsimeq" => (b"tilde.eq.rev", true),
  b"cong" => (b"tilde.equiv", true),
  b"ncong" => (b"tilde.equiv.not", true),
  b"nsim" => (b"tilde.not", true),
  b"backsim" => (b"tilde.rev", true),
  b"hat{}" => (b"hat", true),
  b"prime" => (b"prime", true),
  b"backprime" => (b"prime.rev", true),
  b"bigoplus" => (b"plus.circle.big", true),
  b"dotplus" => (b"plus.dot", true),
  b"boxplus" => (b"plus.square", true),
  b"boxminus" => (b"minus.square", true),
  b"eqsim" => (b"minus.tilde", true),
  b"otimes" => (b"times.circle", true),
  b"bigotimes" => (b"times.circle.big", true),
  b"divideontimes" => (b"times.div", true),
  b"leftthreetimes" => (b"times.three.l", true),
  b"rightthreetimes" => (b"times.three.r", true),
  b"ltimes" => (b"times.l", true),
  b"rtimes" => (b"times.r", true),
  b"boxtimes" => (b"times.square", true),
  b"triangleq" => (b"eq.delta", true),
  b"curlyeqprec" => (b"eq.prec", true),
  b"curlyeqsucc" => (b"eq.succ", true),
  b"gtrdot" => (b"gt.dot", true),
  b"gg" => (b"gt.double", true),
  b"gtreqless" => (b"gt.eq.lt", true),
  b"ngeq" => (b"gt.eq.not", true),
  b"geqq" => (b"gt.equiv", true),
  b"gtrless" => (b"gt.lt", true),
  b"gneqq" => (b"gt.nequiv", true),
  b"ngtr" => (b"gt.not", true),
  b"gnsim" => (b"gt.ntilde", true),
  b"gtrsim" => (b"gt.tilde", true),
  b"vartriangleright" => (b"gt.tri", true),
  b"trianglerighteq" => (b"gt.tri.eq", true),
  b"ntrianglerighteq" => (b"gt.tri.eq.not", true),
  b"ntriangleright" => (b"gt.tri.not", true),
  b"ggg" => (b"gt.triple", true),
  b"lessdot" => (b"lt.dot", true),
  b"ll" => (b"lt.double", true),
  b"lesseqgtr" => (b"lt.eq.gt", true),
  b"nleq" => (b"lt.eq.not", true),
  b"leqq" => (b"lt.equiv", true),
  b"lessgtr" => (b"lt.gt", true),
  b"lneqq" => (b"lt.nequiv", true),
  b"nless" => (b"lt.not", true),
  b"lnsim" => (b"lt.ntilde", true),
  b"lesssim" => (b"lt.tilde", true),
  b"vartriangleleft" => (b"lt.tri", true),
  b"trianglelefteq" => (b"lt.tri.eq", true),
  b"ntrianglelefteq" => (b"lt.tri.eq.not", true),
  b"ntriangleleft" => (b"lt.tri.not", true),
  b"lll" => (b"lt.triple", true),
  b"approxeq" => (b"approx.eq", true),
  b"prec" => (b"prec", true),
  b"precapprox" => (b"prec.approx", true),
  b"preccurlyeq" => (b"prec.eq", true),
  b"npreceq" => (b"prec.eq.not", true),
  b"precnapprox" => (b"prec.napprox", true),
  b"nprec" => (b"prec.not", true),
  b"precnsim" => (b"prec.ntilde", true),
  b"precsim" => (b"prec.tilde", true),
  b"succ" => (b"succ", true),
  b"succapprox" => (b"succ.approx", true),
  b"succcurlyeq" => (b"succ.eq", true),
  b"nsucceq" => (b"succ.eq.not", true),
  b"succnapprox" => (b"succ.napprox", true),
  b"nsucc" => (b"succ.not", true),
  b"succnsim" => (b"succ.ntilde", true),
  b"succsim" => (b"succ.tilde", true),
  b"equiv" => (b"equiv", true),
  b"propto" => (b"prop", true),
  b"varnothing" => (b"nothing", true),
  b"smallsetminus" => (b"without", true),
  b"complement" => (b"complement", true),
  b"ni" => (b"in.rev", true),
  b"Subset" => (b"subset.double", true),
  b"nsubseteq" => (b"subset.eq.not", true),
  b"sqsubseteq" => (b"subset.eq.sq", true),
  b"subsetneq" => (b"subset.neq", true),
  b"supset" => (b"supset", true),
  b"Supset" => (b"supset.double", true),
  b"supseteq" => (b"supset.eq", true),
  b"nsupseteq" => (b"supset.eq.not", true),
  b"sqsupseteq" => (b"supset.eq.sq", true),
  b"supsetneq" => (b"supset.neq", true),
  b"bigcup" => (b"union.big", true),
  b"Cup" => (b"union.double", true),
  b"uplus" => (b"union.plus", true),
  b"biguplus" => (b"union.plus.big", true),
  b"sqcup" => (b"union.sq", true),
  b"bigsqcup" => (b"union.sq.big", true),
  b"bigcap" => (b"sect.big", true),
  b"Cap" => (b"sect.double", true),
  b"sqcap" => (b"sect.sq", true),
  b"partial" => (b"diff", true),
  b"nabla" => (b"nabla", true),
  b"coprod" => (b"product.co", true),
  b"forall" => (b"forall", true),
  b"exists" => (b"exists", true),
  b"nexists" => (b"exists.not", true),
  b"top" => (b"top", true),
  b"bot" => (b"bot", true),
  b"neg" => (b"not", true),
  b"land" => (b"and", true),
  b"bigwedge" => (b"and.big", true),
  b"curlywedge" => (b"and.curly", true),
  b"vee" => (b"or", true),
  b"bigvee" => (b"or.big", true),
  b"curlyvee" => (b"or.curly", true),
  b"models" => (b"models", true),
  b"therefore" => (b"therefore", true),
  b"because" => (b"because", true),
  b"blacksquare" => (b"qed", true),
  b"circ" => (b"compose", true),
  b"multimap" => (b"multimap", true),
  b"mid" => (b"divides", true),
  b"nmid" => (b"divides.not", true),
  b"wr" => (b"wreath", true),
  b"parallel" => (b"parallel", true),
  b"nparallel" => (b"parallel.not", true),
  b"perp" => (b"perp", true),
  b"Join" => (b"join", true),
  b"pounds" => (b"pound", true),
  b"clubsuit" => (b"suit.club", true),
  b"spadesuit" => (b"suit.spade", true),
  b"bullet" => (b"bullet", true),
  b"circledcirc" => (b"circle.nested", true),
  b"rhd" => (b"triangle.stroked.r", true),
  b"lhd" => (b"triangle.stroked.l", true),
  b"bigtriangleup" => (b"triangle.stroked.t", true),
  b"bigtriangledown" => (b"triangle.stroked.b", true),
  b"triangleright" => (b"triangle.stroked.small.r", true),
  b"triangledown" => (b"triangle.stroked.small.b", true),
  b"triangleleft" => (b"triangle.stroked.small.l", true),
  b"vartriangle" => (b"triangle.stroked.small.t", true),
  b"square" => (b"square.stroked", true),
  b"Diamond" => (b"diamond.stroked", true),
  b"diamond" => (b"diamond.stroked.small", true),
  b"lozenge" => (b"lozenge.stroked", true),
  b"blacklozenge" => (b"lozenge.filled", true),
  b"bigstar" => (b"star.stroked", true),
  b"longmapsto" => (b"arrow.r.long.bar", true),
  b"nRightarrow" => (b"arrow.r.double.not", true),
  b"hookrightarrow" => (b"arrow.r.hook", true),
  b"looparrowright" => (b"arrow.r.loop", true),
  b"nrightarrow" => (b"arrow.r.not", true),
  b"rightsquigarrow" => (b"arrow.r.squiggly", true),
  b"rightarrowtail" => (b"arrow.r.tail", true),
  b"Rrightarrow" => (b"arrow.r.triple", true),
  b"twoheadrightarrow" => (b"arrow.r.twohead", true),
  b"nLeftarrow" => (b"arrow.l.double.not", true),
  b"hookleftarrow" => (b"arrow.l.hook", true),
  b"looparrowleft" => (b"arrow.l.loop", true),
  b"nleftarrow" => (b"arrow.l.not", true),
  b"leftarrowtail" => (b"arrow.l.tail", true),
  b"Lleftarrow" => (b"arrow.l.triple", true),
  b"twoheadleftarrow" => (b"arrow.l.twohead", true),
  b"uparrow" => (b"arrow.t", true),
  b"Uparrow" => (b"arrow.t.double", true),
  b"downarrow" => (b"arrow.b", true),
  b"Downarrow" => (b"arrow.b.double", true),
  b"nLeftrightarrow" => (b"arrow.l.r.double.not", true),
  b"nleftrightarrow" => (b"arrow.l.r.not", true),
  b"leftrightsquigarrow" => (b"arrow.l.r.wave", true),
  b"updownarrow" => (b"arrow.t.b", true),
  b"Updownarrow" => (b"arrow.t.b.double", true),
  b"nearrow" => (b"arrow.tr", true),
  b"searrow" => (b"arrow.br", true),
  b"nwarrow" => (b"arrow.tl", true),
  b"swarrow" => (b"arrow.bl", true),
  b"circlearrowleft" => (b"arrow.ccw", true),
  b"curvearrowleft" => (b"arrow.ccw.half", true),
  b"circlearrowright" => (b"arrow.cw", true),
  b"curvearrowright" => (b"arrow.cw.half", true),
  b"rightrightarrows" => (b"arrows.rr", true),
  b"leftleftarrows" => (b"arrows.ll", true),
  b"upuparrows" => (b"arrows.tt", true),
  b"downdownarrows" => (b"arrows.bb", true),
  b"leftrightarrows" => (b"arrows.lr", true),
  b"rightleftarrows" => (b"arrows.rl", true),
  b"rightharpoonup" => (b"harpoon.rt", true),
  b"rightharpoondown" => (b"harpoon.rb", true),
  b"leftharpoonup" => (b"harpoon.lt", true),
  b"leftharpoondown" => (b"harpoon.lb", true),
  b"upharpoonleft" => (b"harpoon.tl", true),
  b"upharpoonright" => (b"harpoon.tr", true),
  b"downharpoonleft" => (b"harpoon.bl", true),
  b"downharpoonright" => (b"harpoon.br", true),
  b"leftrightharpoons" => (b"harpoons.ltrb", true),
  b"rightleftharpoons" => (b"harpoons.rtlb", true),
  b"vdash" => (b"tack.r", true),
  b"nvdash" => (b"tack.r.not", true),
  b"vDash" => (b"tack.r.double", true),
  b"nvDash" => (b"tack.r.double.not", true),
  b"dashv" => (b"tack.l", true),
  b"varepsilon" => (b"epsilon", true),
  b"varphi" => (b"phi", true),
  b"varpi" => (b"pi.alt", true),
  b"varrho" => (b"rho.alt", true),
  b"varsigma" => (b"sigma.alt", true),
  b"vartheta" => (b"theta.alt", true),
  b"ell" => (b"ell", true),
  b"hslash" => (b"planck.reduce", true),
  b"Re" => (b"Re", true),
  b"Im" => (b"Im", true),
  b"imath" => (b"dotless.i", true),
  b"jmath" => (b"dotless.j", true),
  // Ignores
  b"displaystyle" => (b"", false),
  b"textstyle" => (b"", false),
  b"scriptstyle" => (b"", false),
  b"scriptscriptstyle" => (b"", false),
  b"bf" => (b"", false),
  b"rm" => (b"", false),
  b"it" => (b"", false),
  b"sf" => (b"", false),
  b"frak" => (b"", false),
  b"tt" => (b"", false),
  b"cal" => (b"", false),
  b"limits" => (b"", false),
  b"nolimits" => (b"", false),
};

#[wasm_func]
pub fn convert(input: &[u8]) -> Result<Vec<u8>, String> {
  // mutable Vec<u8> to store the output
  let mut output: Vec<u8> = Vec::new();
  // count for detect whether the mode is matrix
  let mut matrix_count = 0;
  // loop to eat the input
  let mut i = 0;
  while i < input.len() {
    // special handle for & and \\
    if input[i] == b'&' {
      if matrix_count > 0 {
        output.extend_from_slice(b", ");
      } else {
        output.extend_from_slice(b"& ");
      }
      i += 1;
      continue;
    } else if input[i] == b'\\' && i+1 < input.len() && input[i+1] == b'\\' {
      if matrix_count > 0 {
        output.extend_from_slice(b"; ");
      } else {
        output.extend_from_slice(b"\\ ");
      }
      i += 2;
      continue;
    }
    // find the first match
    let mut matched = false;
    for (regex, replacement, map_type) in TOKEN_MAP_LIST.iter() {
      if let Some(m) = regex.captures(&input[i..]) {
        // map the matched string
        match map_type {
          MapType::Skip => {},
          MapType::Normal => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
          },
          MapType::NoSpace => {
            output.extend_from_slice(replacement);
          },
          MapType::SqrtN => {
            output.extend_from_slice(b"root(");
            // group 1
            output.extend_from_slice(&input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()]);
            output.extend_from_slice(b", ");
          },
          MapType::MatrixBegin => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
            matrix_count += 1;
          },
          MapType::MatrixEnd => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
            matrix_count -= 1;
            if matrix_count < 0 {
              return Result::Err(String::from(format!("matrix environment end without begin at {}", i)));
            }
          },
          MapType::Text => {
            // group 1
            let text = &input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()];
            if replacement != b"" {
              // \text{}
              output.extend_from_slice(replacement);
              output.extend_from_slice(b"\"");
              output.extend_from_slice(text);
              output.extend_from_slice(b"\"");
              output.extend_from_slice(b")");
            } else {
              // \operatorname{}
              output.extend_from_slice(b"\"");
              output.extend_from_slice(text);
              output.extend_from_slice(b"\"");
            }
          },
          MapType::Command => {
            // COMMAND_MAP[group 1]
            let key = &input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()];
            if let Some((replacement, add_space)) = COMMAND_MAP.get(key) {
              output.extend_from_slice(*replacement);
              if *add_space {
                output.extend_from_slice(b" ");
              }
            } else {
              return Result::Err(String::from(format!("invalid command \"\\{}\" at {}", String::from_utf8_lossy(key), i)));
            }
          },
          MapType::Default => {
            // group 0
            output.extend_from_slice(&input[i..i+m.get(0).unwrap().end()]);
            output.extend_from_slice(b" ");
          },
        }
        // move the index
        i += m.get(0).unwrap().end();
        matched = true;
        break;
      }
    }
    if !matched {
      // if not matched, panic
      return Result::Err(String::from(format!("no matched pattern for \"{}\"", String::from_utf8_lossy(&input[i..]))));
    }
  }
  Result::Ok(output)
}


// test with b"abc"
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert() {
    println!("{}", String::from_utf8_lossy(&convert(br"\left[\int_1^2 x \mathrm{d} x\right]").unwrap()));
    assert_eq!(convert(br"abc").unwrap(), b"a b c ");
  }
}
