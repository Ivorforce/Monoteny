import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


def main():
    """
    <DOCSTRING TODO>
    """
    print("Test: \"Success\"")
    value = square(float32(2.2)) * math.sin(e())
    print(op.add("Value: ", str(value)))
    b = square(value) + pi_ish_0()
    b = floor_div(square(b) ** (-float32(2.2)), value) + tau()
    c = pi_ish_1() // int32(2)
    print(op.add("Bool Value: ", str(((b < float32(2)) and (not (value > float32(2)))) and True)))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def tau() -> float32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return float32(6.283185307179586)


def floor_div(lhs: float32, rhs: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO

    Returns:
        <TODO>
    """
    return math.floor(lhs / rhs)


def square(self: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return self * self


def e() -> float32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return float32(2.718281828459045)


def pi_ish_0() -> float32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return float32(3.14)


def pi_ish_1() -> int32:
    """
    <DOCSTRING TODO>

    Returns:
        <TODO>
    """
    return int32(3)


__all__ = [
    "main",
]


if __name__ == "__main__":
    main()
