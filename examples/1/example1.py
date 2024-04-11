import numpy as np
import math
import operator as op
from dataclasses import dataclass
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


def main():
    """
    <DOCSTRING TODO>
    """
    print("Test: \"Success\"")
    value: float32 = square(float32(2.2)) * math.sin(e)
    print("Value: " + str(value))
    b: float32 = square(value) + pi_ish
    b: float32 = floor_div(square(b) ** (-float32(2.2)), value) + tau
    print("Bool Value: " + str(((b < float32(2)) and (not (value > float32(2)))) and True))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


pi_ish: float32 = float32(3.14)


def floor_div(lhs: float32, rhs: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO

    Returns:
        <TODO>
    """
    return math.floor(lhs // rhs)


tau: float32 = float32(6.283185307179586)


e: float32 = float32(2.718281828459045)


def square(self: float32) -> float32:
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
