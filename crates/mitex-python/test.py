"""
Tests for mitex_python module using pytest.
"""
import mitex_python

def test_basic_fraction():
    """Test conversion of a basic fraction."""
    result = mitex_python.convert_latex_math("\\frac{1}{2}")
    # Normalize whitespace for comparison
    normalized_result = result.replace(" ", "")
    assert "frac(1,2)" in normalized_result

def test_complex_expression():
    """Test conversion of a more complex expression."""
    latex = "\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}"
    result = mitex_python.convert_latex_math(latex)
    # integral _(0 )^(oo ) e ^(- x ^(2 )) d x  =  frac(mitexsqrt(pi ),2 )
    # Just verify it runs without error and returns a non-empty string
    assert result and isinstance(result, str)

def test_math_operators():
    """Test conversion of common math operators."""
    result = mitex_python.convert_latex_math("a + b - c \\times d \\div e")
    assert "+" in result
    assert "-" in result
    assert "times" in result or "*" in result
    assert "div" in result or "/" in result

def test_error_handling():
    """Test handling of malformed input."""
    # This should not crash but might return an error message or do partial conversion
    result = mitex_python.convert_latex_math("\\frac{1}{")
    assert isinstance(result, str)

if __name__ == "__main__":
    import pytest
    pytest.main([__file__, "-v"])