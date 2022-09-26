import tenlang as tl
import numpy as np
import operator as op
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool
from typing import Any, Callable


def test_1() -> float32:
    """
    <Docstring TODO!>

    Returns: float32
    """
    return float32(5)


def test_0() -> float64:
    """
    <Docstring TODO!>

    Returns: float64
    """
    return float64(2)


def main():
    """
    <Docstring TODO!>
    """
    a = float64(5) + float64(2)
    a = test_0()
    b = a
    print(b)


if __name__ == '__main__':
    main()
