import numpy as np
import math
import operator as op
from dataclasses import dataclass
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


def main():
    """
    <DOCSTRING TODO>
    """
    print(talk_1)
    print(talk_0)
    converse_0()
    converse_1()


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


def converse_0():
    """
    <DOCSTRING TODO>
    """
    print("Conversation: \n    " + (talk_0 + (" \n    " + talk_1)))


def converse_1():
    """
    <DOCSTRING TODO>
    """
    print("Conversation: \n    " + (talk_1 + (" \n    " + talk_0)))


talk_0: str = "Meow"


talk_1: str = "Bark"


__all__ = [
    "main",
]


if __name__ == "__main__":
    main()
