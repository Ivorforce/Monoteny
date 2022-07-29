import numpy as np


def no_op():
    """
    <Docstring TODO!>
    """
    return


def square(a: np.int32,) -> np.int32:
    """
    <Docstring TODO!>

    Args:
        a: np.int32

    Returns: np.int32
    """
    return (a) * (a)


def main():
    """
    <Docstring TODO!>
    """
    b = np.int32(5)
    b = np.int32(3)
    print("b: ")
    print((np.int32(5)) > (((np.int32(2)) ** (np.int32(3))) * (np.int32(2))))
    print((False) | (True))
    print([b, square(b), np.int32(2)])
