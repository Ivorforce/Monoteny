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
    print(talk_1)
    print(talk_0)
    converse_1()
    converse_0()


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


talk_0: str = "Meow"


talk_1: str = "Bark"


def converse_0():
    """
    <DOCSTRING TODO>
    """
    print("Conversation: \n    " + (talk_1 + (" \n    " + talk_0)))


def converse_1():
    """
    <DOCSTRING TODO>
    """
    print("Conversation: \n    " + (talk_0 + (" \n    " + talk_1)))


def main_():
    """
    <DOCSTRING TODO>
    """
    dog: Dog = Dog()
    cat: Cat = Cat()
    print(talk_1)
    print(talk_0)
    converse_1()
    converse_0()


__all__ = [
    "Cat",
    "Dog",
    "main",
]


if __name__ == "__main__":
    main()
