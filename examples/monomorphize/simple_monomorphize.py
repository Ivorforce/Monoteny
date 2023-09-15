import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


def main():
    """
    <DOCSTRING TODO>
    """
    write_line_0(square_1(int32("3.")))
    write_line_1(square_0(float32(3.2)))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def square_0(self: float32) -> float32:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return self * self


def write_line_0(value: int32):
    """
    <DOCSTRING TODO>

    Args:
        value: TODO
    """
    print(str(value))


def write_line_1(value: float32):
    """
    <DOCSTRING TODO>

    Args:
        value: TODO
    """
    print(str(value))


def square_1(self: int32) -> int32:
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
