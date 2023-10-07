import numpy as np
import math
import operator as op
from dataclasses import dataclass
from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
from typing import Any, Callable


@dataclass
class Animal:
    species: str
    name: str
    height_cm: float32


def main():
    """
    <DOCSTRING TODO>
    """
    animal: Animal = Animal(species="Cat", name="Noir", height_cm=float32(180))
    print(animal.name + (" (" + (animal.species + (") was: " + (str(animal.height_cm) + "cm")))))


# ========================== ======== ============================
# ========================== Internal ============================
# ========================== ======== ============================


__all__ = [
    "Animal",
    "main",
]


if __name__ == "__main__":
    main()
