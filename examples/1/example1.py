import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def main():
    """
    <DOCSTRING TODO>
    """
    print("Test: ")
    value = square_2(float32(2.2)) * math.sin(e())
    b = square_0(value=value) + pi_ish_0()
    b = floor_div(square_3(b) ** (-float32(2.2)), value) + tau()
    c = pi_ish_1()
    print(((b < float32(2)) and (not (value > float32(2)))) and True)


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def pi_ish_0() -> float32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return float32(3.14)


def square_0(value: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        value: TODO

    Returns:
        <TODO>
    """
    return square_1(value)


def square_1(self: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return self * self


def floor_div(lhs: float32, rhs: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO

    Returns:
        <TODO>
    """
    return lhs / rhs


def square_2(self: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return self * self


def pi_ish_1() -> int32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return int32(3)


def tau() -> float32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return float32(6.283185307179586)


def e() -> float32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return float32(2.718281828459045)


def square_3(self: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return self * self


__all__ = [
    "main",
]


if __name__ == "__main__":
    main()
