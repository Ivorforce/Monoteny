import numpy as np
from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool


def floor_div(lhs: int32,rhs: int32,) -> int32:
    """
    <Docstring TODO!>

    Args:
        lhs: int32
        rhs: int32

    Returns: int32
    """
    return lhs / rhs


def no_op():
    """
    <Docstring TODO!>
    """
    pass


def square_0(a: int32,) -> int32:
    """
    <Docstring TODO!>

    Args:
        a: int32

    Returns: int32
    """
    return a * a


def square_1(self: int32,) -> int32:
    """
    <Docstring TODO!>

    Args:
        self: int32

    Returns: int32
    """
    return self * self


def main():
    """
    <Docstring TODO!>
    """
    b = int32(5)
    b = int32(3)
    no_op()
    print("b: ")
    print((int32(5)) > (floor_div(int32(2) ** int32(3), int32(2))) and (floor_div(int32(2) ** int32(3), int32(2))) > (-int32(2)))
    print((int32(5) > int32(2)) | True)
    print((int32(2) ** (int32(3) ** int32(5))) + int32(2))
    print([b, square_0(b), square_1(int32(2))])
