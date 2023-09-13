import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


class Cat:
    pass


class Dog:
    pass


def main():
    """
    <DOCSTRING TODO>
    """
    dog = Dog()
    cat = Cat()
    converse_1(cat, dog)
    converse_0(dog, cat)


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def converse_0(lhs: Dog, rhs: Cat):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    talk_1(lhs)
    talk_0(rhs)


def talk_0(self: Cat):
    """
    <DOCSTRING TODO>

    Args:
        self: TODO
    """
    print("Meow")


def talk_1(self: Dog):
    """
    <DOCSTRING TODO>

    Args:
        self: TODO
    """
    print("Bark")


def converse_1(lhs: Cat, rhs: Dog):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    talk_0(lhs)
    talk_1(rhs)


__all__ = [
    "Cat",
    "Dog",
    "main",
]


if __name__ == "__main__":
    main()
