import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def main():
    """
    <Docstring TODO!>
    """
    print("Test: ")
    a = (square_0(int64(2), multiply=op.mul)) * int64(3)
    b = square_1(float64(5.1), multiply=op.mul)
    b = floor_div((square_0(b, multiply=op.mul)) ** float64(2.2), float64(2), divide=op.truediv)
    print(a)
    print(b)


def floor_div(__0: Any, __1: Any, divide: Callable, subtract=None, add=None, multiply=None, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any
        __1: Any

    Returns: Any
    """
    lhs = __0
    rhs = __1
    return divide(lhs, rhs)


def square_0(__0: Any, multiply: Callable, divide=None, subtract=None, add=None, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    self = __0
    return multiply(self, self)


def square_1(__0: Any, multiply: Callable, add=None, subtract=None, divide=None, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    a = __0
    return multiply(a, a)
