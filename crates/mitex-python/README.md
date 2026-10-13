# mitex-python

Python bindings for the mitex LaTeX to Typst converter.

## Description

`mitex-python` provides Python bindings for the [mitex](https://github.com/mitex-rs/mitex) library, a fast and reliable LaTeX to Typst converter written in Rust. This package allows Python developers to easily convert LaTeX math expressions to Typst syntax.

## Features

- Convert LaTeX math expressions to Typst syntax
- High performance through Rust implementation
- Seamless Python integration via PyO3

## Installation

Currently only available by building from source. See the Development section below.

## Usage

```python
import mitex_python

# Convert a simple LaTeX expression
result = mitex_python.convert_latex_math("\\frac{1}{2}")
print(result)  # Output: "$frac(1, 2)$"

# Convert a more complex expression
complex_expr = "\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}"
result = mitex_python.convert_latex_math(complex_expr)
print(result)
```

## Development

To build and install `mitex-python`, we need few things.

Prerequisites:
- Rust toolchain
- Python 3.8+

First, navigate to the project directory:
```
cd crates/mitex-python
```

The building needs `maturin`
```
pip install -r requirements.txt
# or
pip install maturin
```

To install `mitex-python`, 
```
cd crates/mitex-python  &&  make develop
```

Common commands (from `crates/mitex-python`):
- Install: `make` or `make develop`

- `make release`: Builds an optimized version and installs it directly in your current Python environment (using `maturin develop --release`)
- `make wheel`: Only builds a wheel file without installing it (using `maturin build`). Creates unoptimized debug wheels in target/wheels
- `make wheel-release`: Only builds an optimized wheel file without installing it (using `maturin build --release`). Creates release/optimized wheels in target/wheels
- `make test` : Run tests (needs `pytest`)


You can also call the script directly:
- `python3 crates/mitex-python/build.py [--release] [--wheel] [--test]`