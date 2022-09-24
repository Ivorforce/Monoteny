import tenlang as tl
import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def floor_div(lhs: Any, rhs: Any, Float: tl.traits.Float, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        lhs: Any
        rhs: Any

    Returns: Any
    """
    return Float.divide(lhs, rhs)


def square_0(a: Any, Number: tl.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        a: Any

    Returns: Any
    """
    return Number.multiply(a, a)


def square_1(self: Any, Number: tl.traits.Number, ) -> Any:
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
    a = (square_1(int64(2), Number=tl.declarations.Number_3)) * int64(3)
    b = square_0(float64(5.1), Number=tl.declarations.Number_11)
    b = ((floor_div((square_1(b, Number=tl.declarations.Number_11)) ** float64(2.2), float64(2), Float=tl.declarations.Float_1)) > float64(2.3)) | (int64(5) > int64(2))
    print(a)
    print(b)


if __name__ == '__main__':
    main()
