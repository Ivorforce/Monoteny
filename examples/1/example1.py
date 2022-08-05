import numpy as np


def no_op():
    """
    <Docstring TODO!>
    """
    pass


def square_a(a: np.int32,) -> np.int32:
    """
    <Docstring TODO!>

    Args:
        a: np.int32

    Returns: np.int32
    """
    return a * a


def square_b(self: np.int32,) -> np.int32:
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
    print("b: ")
    print((np.int32(5)) > ((np.int32(2) ** np.int32(3)) * np.int32(2)) and ((np.int32(2) ** np.int32(3)) * np.int32(2)) > (-np.int32(2)))
    print((np.int32(5) > np.int32(2)) | True)
    print(((np.int32(2) ** (np.int32(3) ** np.int32(5))) * np.int32(3)) + np.int32(5))
    print([b, square_a(b), square_b(np.int32(2))])
