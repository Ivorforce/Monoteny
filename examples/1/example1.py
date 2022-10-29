import monoteny as mn
import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def floor_div(lhs: Any, rhs: Any, Float: mn.traits.Float, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        lhs: Any
        rhs: Any

    Returns: Any
    """
    return Float.divide(lhs, rhs)


def square_1(a: Any, Number: mn.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        a: Any

    Returns: Any
    """
    return Number.multiply(a, a)


def square_0(self: Any, Number: mn.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        self: Any

    Returns: Any
    """
    return square_1(self, Number=Number)


def pi_ish_0(Float: mn.traits.Float, ) -> Any:
    """
    <Docstring TODO!>

    Returns: Any
    """
    return Float.parse_float_literal("3.14")


def pi_ish_1(Int: mn.traits.Int, ) -> Any:
    """
    <Docstring TODO!>

    Returns: Any
    """
    return Int.parse_int_literal("3")


def main():
    """
    <Docstring TODO!>
    """
    print("Test: ")
    a = (square_1(float32(2.2), Number=mn.declarations.Number_10)) * (float32(np.e))
    b = (square_1(float32(5), Number=mn.declarations.Number_10)) + (pi_ish_0(Float=mn.declarations.Float_0))
    b = floor_div((square_1(b, Number=mn.declarations.Number_10)) ** (-(float32(2.2))), math.log(a, float32(np.pi * 2)), Float=mn.declarations.Float_0)
    c = pi_ish_1(Int=mn.declarations.Int_2)
    print(((b < (float32(2))) and (not (a > (float32(2))))) and (True))


if __name__ == '__main__':
    main()
