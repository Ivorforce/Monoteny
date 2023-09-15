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
    print_(talk_0(dog))
    print_(talk_1(cat))
    converse_0(cat, dog)
    converse_1(dog, cat)


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def talk_0(self: Dog) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return "Bark"


def converse_0(lhs: Cat, rhs: Dog):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    print_(op.add(format(talk_1(lhs)), op.add(" ", format(talk_0(rhs)))))


def format(object: str) -> str:
    """
    <DOCSTRING TODO>

    Args:
        object: TODO

    Returns:
        <TODO>
    """
    return to_string(object)


def to_string(self: str) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return self


def talk_1(self: Cat) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return "Meow"


def print_(value: str):
    """
    <DOCSTRING TODO>

    Args:
        value: TODO
    """
    print(to_string(value))


def converse_1(lhs: Dog, rhs: Cat):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    print_(op.add(format(talk_0(lhs)), op.add(" ", format(talk_1(rhs)))))


__all__ = [
    "Cat",
    "Dog",
    "main",
]


if __name__ == "__main__":
    main()
