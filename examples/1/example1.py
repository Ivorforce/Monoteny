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
    Float.divide(lhs, rhs)


def square_0(a: Any, Number: tl.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        a: Any

    Returns: Any
    """
    Number.multiply(a, a)


def square_1(self: Any, Number: tl.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        self: Any

    Returns: Any
    """
    square_0(self, Number=Number)


def main():
    """
    <Docstring TODO!>
    """
    print("Test: ")
    a = (square_1(float32(2), Number=tl.declarations.Number_10)) * float32(3)
    b = square_0(float32(5.1), Number=tl.declarations.Number_10)
    b = floor_div((square_1(b, Number=tl.declarations.Number_10)) ** float32(2.2), float32(2), Float=tl.declarations.Float_0)
    print(a)
    print((b < float32(2)) | (a > float32(2)))


if __name__ == '__main__':
    main()
