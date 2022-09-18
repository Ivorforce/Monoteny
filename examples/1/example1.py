import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def square_0(__0: Any, multiply: Callable, modulo=None, is_greater=None, is_equal=None, is_greater_or_equal=None, is_not_equal=None, subtract=None, divide=None, is_lesser_or_equal=None, is_lesser=None, negative=None, positive=None, add=None, ) -> Any:
    """
    <Docstring TODO!>

    Args:
        __0: Any

    Returns: Any
    """
    a = __0
    return multiply(a, a)


def floor_div(__0: Any, __1: Any, divide: Callable, positive=None, modulo=None, is_lesser_or_equal=None, is_lesser=None, subtract=None, negative=None, is_greater=None, is_equal=None, is_not_equal=None, is_greater_or_equal=None, add=None, multiply=None, ) -> Any:
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


def square_1(__0: Any, multiply: Callable, subtract=None, add=None, negative=None, modulo=None, is_lesser=None, positive=None, is_greater_or_equal=None, is_lesser_or_equal=None, is_greater=None, is_equal=None, is_not_equal=None, divide=None, ) -> Any:
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
    print("Test: ")
    a = (square_1(int64(2), multiply=op.mul)) * int64(3)
    b = square_0(float64(5.1), multiply=op.mul)
    b = ((floor_div((square_1(b, multiply=op.mul)) ** float64(2.2), float64(2), divide=op.truediv)) > float64(2.3)) | (int64(5) > int64(2))
    print(a)
    print(b)
