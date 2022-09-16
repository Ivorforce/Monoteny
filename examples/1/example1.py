import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def square_0(__0: Any, multiply: Callable, subtract=None, add=None, divide=None, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    a = __0
    return multiply(a, a)


def floor_div(__0: Any, __1: Any, divide: Callable, add=None, subtract=None, multiply=None, ) -> Any:
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


def main():
    """
    <Docstring TODO!>
    """
    b = int32(5)
    b = int32(3)
    print("b: ")
    print(int32(5) > (floor_div(int32(2) ** int32(3), int32(2), divide=op.truediv)))
    print((int32(5) > int32(2)) | True)
    print((int32(2) ** (int32(3) ** int32(5))) + int32(2))
    print([b, square_0(b, multiply=op.mul), square_1(int32(2), multiply=op.mul)])


def square_1(__0: Any, multiply: Callable, divide=None, subtract=None, add=None, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    self = __0
    return multiply(self, self)
