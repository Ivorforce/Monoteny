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
    value: float32 = square(float32(2.2))
    print("Value: " + str(value))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


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
