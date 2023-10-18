import numpy as np
import math
import operator as op
from dataclasses import dataclass
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


@dataclass
class Dog:
    pass


@dataclass
class Cat:
    pass


def main():
    """
    <DOCSTRING TODO>
    """
    dog: Dog = Dog()
    cat: Cat = Cat()
    print(talk_1(dog))
    print(talk_0(cat))
    converse_1(cat, dog)
    converse_0(dog, cat)


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def talk_0(self: Cat) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return "Meow"


def talk_1(self: Dog) -> str:
    """
    <DOCSTRING TODO>

    Args:
        self: TODO

    Returns:
        <TODO>
    """
    return "Bark"


def converse_0(lhs: Dog, rhs: Cat):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    print("Conversation: \n    " + (talk_1(lhs) + (" \n    " + talk_0(rhs))))


def converse_1(lhs: Cat, rhs: Dog):
    """
    <DOCSTRING TODO>

    Args:
        lhs: TODO
        rhs: TODO
    """
    print("Conversation: \n    " + (talk_0(lhs) + (" \n    " + talk_1(rhs))))


__all__ = [
    "Cat",
    "Dog",
    "main",
]


if __name__ == "__main__":
    main()
