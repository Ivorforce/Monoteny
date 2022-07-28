# TenLang

Welcome to the mathemagical land of TenLang! 

TenLang is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in functional and object-oriented languages. Here are some of the main philosophies:

- Every object and object collection is a broadcastable NDArray.
  - That includes arrays, sets, dictionaries and tuples.
- The compiler guarantees NDArray broadcast and lookup safety.
- All actions on an NDArray broadcast to their atoms, unless specified.
  - Yes, that includes function calls and member lookups.

## Transpilation Targets

Many excellent tools and complex infrastructure exists in other ecosystems. As a single-feature language, TenLang does not aim to build a new ecosystem.

Instead, TenLang transpiles to insert into these ecosystems. Hereby, any frameworks built in TenLang will be available in *all* of the ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] C++ with Eigen
* [Future] Octave / MatLab
* [Future] R

## Roadmap

- [x] Project Skeleton
- [x] Primitives
- [x] Function Interfaces
- [x] Int call / subscript keying
- [ ] Type Inheritance
- [ ] Generic Type Inferral
- [ ] Overloading & Call Type Checking
- [ ] Tuples
- [ ] Structs
- [ ] Interfaces
- [ ] Extensions
- [ ] Exceptions
- [ ] Dynamic calls / subscript
- [ ] Optionals
- [ ] Dynamic Shapes/ Optional Lookups
- [ ] Staggered Dimensions
- [ ] Dictionaries
- [ ] Sets
- [ ] Implicit tensor building
- [ ] Int NDArrays
- [ ] Auto Broadcast
- [ ] Standard Library
- [ ] System Callback API / Permission Contexts
