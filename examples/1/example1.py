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


def square_0(a: Any, Number: mn.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        a: Any

    Returns: Any
    """
    return Number.multiply(a, a)


def square_1(self: Any, Number: mn.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        self: Any

    Returns: Any
    """
    return square_0(self, Number=Number)


def main():
    """
    <Docstring TODO!>
    """
    print("Test: ")
    a = (square_0(float32(2.2), Number=mn.declarations.Number_10)) * float32(3)
    b = square_0(float32(5), Number=mn.declarations.Number_10)
    b = floor_div((square_0(b, Number=mn.declarations.Number_10)) ** (-float32(2.2)), math.log(a, float32(2)), Float=mn.declarations.Float_0)
    print(((b < float32(2)) and (not (a > float32(2)))) and True)


if __name__ == '__main__':
    main()
