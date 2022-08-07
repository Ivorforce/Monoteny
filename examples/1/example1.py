import numpy as np


def floor_div(lhs: np.int32,rhs: np.int32,) -> np.int32:
    """
    <Docstring TODO!>

    Args:
        lhs: np.int32
        rhs: np.int32

    Returns: np.int32
    """
    return lhs / rhs


def no_op():
    """
    <Docstring TODO!>
    """
    pass


def square_0(a: np.int32,) -> np.int32:
    """
    <Docstring TODO!>

    Args:
        a: np.int32

    Returns: np.int32
    """
    return a * a


def square_1(self: np.int32,) -> np.int32:
    """
    <Docstring TODO!>

    Args:
        self: np.int32

    Returns: np.int32
    """
    return self * self


def main():
    """
    <Docstring TODO!>
    """
    b = np.int32(5)
    b = np.int32(3)
    no_op()
    print("b: ")
    print((np.int32(5)) > (floor_div(np.int32(2) ** np.int32(3), np.int32(2))) and (floor_div(np.int32(2) ** np.int32(3), np.int32(2))) > (-np.int32(2)))
    print((np.int32(5) > np.int32(2)) | True)
    print(((np.int32(2) ** (np.int32(3) ** np.int32(5))) * np.int32(3)) + np.int32(5))
    print([b, square_0(b), square_1(np.int32(2))])
