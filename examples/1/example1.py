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
    return (a * a)


def main():
    """
    <Docstring TODO!>
    """
    b = np.int32(5)
    print("b: ",)
    print(b,)
    print([b,square(a=b,),np.int32(2),],)
