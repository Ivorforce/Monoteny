import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def main():
    """
    <Docstring TODO!>
    """
    print("Test: ")
    value = (square_2(float32(2.2))) * (e())
    b = (square_0(value=value)) + (pi_ish_1())
    b = (floor_div((square_3(b)) ** (-(float32(2.2))), value)) + (tau())
    c = pi_ish_0()
    print(((b < (float32(2))) and (not (value > (float32(2))))) and (True))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def square_0(value: float32, ) -> float32:
    """
    <Docstring TODO!>

    Args:
        value: float32

    Returns: float32
    """
    return square_1(value)


def square_1(self: float32, ) -> float32:
    """
    <Docstring TODO!>

    Args:
        self: float32

    Returns: float32
    """
    return self * self


def pi_ish_0() -> int32:
    """
    <Docstring TODO!>

    Returns: int32
    """
    return int32(3)


def pi_ish_1() -> float32:
    """
    <Docstring TODO!>

    Returns: float32
    """
    return float32(3.14)


def square_2(self: float32, ) -> float32:
    """
    <Docstring TODO!>

    Args:
        self: float32

    Returns: float32
    """
    return self * self


def floor_div(lhs: float32, rhs: float32, ) -> float32:
    """
    <Docstring TODO!>

    Args:
        lhs: float32
        rhs: float32

    Returns: float32
    """
    return lhs / rhs


def tau() -> float32:
    """
    <Docstring TODO!>

    Returns: float32
    """
    return float32(6.283185307179586)


def e() -> float32:
    """
    <Docstring TODO!>

    Returns: float32
    """
    return float32(2.718281828459045)


def square_3(self: float32, ) -> float32:
    """
    <Docstring TODO!>

    Args:
        self: float32

    Returns: float32
    """
    return self * self


__all__ = [
    "main",
]


if __name__ == "__main__":
    main()
