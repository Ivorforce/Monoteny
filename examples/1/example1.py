import tenlang as tl
import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


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


def square_0(__0: Any, Number: tl.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    a = __0
    return Number.multiply(a, a)


def floor_div(__0: Any, __1: Any, Float: tl.traits.Float, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any
        __1: Any

    Returns: Any
    """
    lhs = __0
    rhs = __1
    return Float.divide(lhs, rhs)


def square_1(__0: Any, Number: tl.traits.Number, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    self = __0
    return square_0(self, Number=Number)
