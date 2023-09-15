import numpy as np
import math
import operator as op
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


class Dog:
    pass


class Cat:
    pass


def main():
    """
    <DOCSTRING TODO>
    """
    dog = Dog()
    cat = Cat()
    print(talk_1(dog))
    print(talk_0(cat))
    converse_0(cat, dog)
    converse_1(dog, cat)


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def converse_0(lhs: Cat, rhs: Dog):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    print(op.add(talk_0(lhs), op.add(" ", talk_1(rhs))))


def talk_0(self: Cat) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return "Meow"


def converse_1(lhs: Dog, rhs: Cat):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    print(op.add(talk_1(lhs), op.add(" ", talk_0(rhs))))


def talk_1(self: Dog) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return "Bark"


__all__ = [
    "Cat",
    "Dog",
    "main",
]


if __name__ == "__main__":
    main()
