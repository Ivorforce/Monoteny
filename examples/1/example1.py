import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def no_op():
    """
    <Docstring TODO!>
    """
    pass


def floor_div(__0: Any, __1: Any, divide: Callable, subtract: Callable, multiply: Callable, add: Callable, ) -> Any:
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


def square_0(__0: Any, add: Callable, divide: Callable, multiply: Callable, subtract: Callable, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    self = __0
    return multiply(self, self)


def main():
    """
    <Docstring TODO!>
    """
    b = int32(5)
    b = int32(3)
    no_op()
    print("b: ")
    print(int32(5) > (floor_div(int32(2) ** int32(3), int32(2), multiply=op.mul, add=op.add, subtract=op.sub, divide=op.truediv)))
    print((int32(5) > int32(2)) | True)
    print((int32(2) ** (int32(3) ** int32(5))) + int32(2))
    print([b, square_1(b, add=op.add, subtract=op.sub, multiply=op.mul, divide=op.truediv), square_0(int32(2), multiply=op.mul, subtract=op.sub, add=op.add, divide=op.truediv)])


def square_1(__0: Any, add: Callable, subtract: Callable, divide: Callable, multiply: Callable, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    a = __0
    return multiply(a, a)
