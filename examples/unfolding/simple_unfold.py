import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def main():
    """
    <Docstring TODO!>
    """
    print(square_0(int32(3)))
    print(square_1(float32(3.2)))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def square_0(self: int32, ) -> int32:
    """
    <Docstring TODO!>

    Args:
        self: int32

    Returns: int32
    """
    return self * self


def square_1(self: float32, ) -> float32:
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