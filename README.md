# TenLang

Welcome to the mathemagical land of TenLang! 

TenLang is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in functional and object-oriented languages. Here are some of the main philosophies:

- Every object and object collection is a broadcastable NDArray.
  - That includes arrays, sets, dictionaries and tuples.
- The compiler guarantees NDArray broadcast and lookup safety.
- All actions on an NDArray broadcast to their atoms, unless specified.
  - Yes, that includes function calls and member lookups.

## Transpilation Targets

TenLang lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things. Therefore, TenLang aims to focus on its most vital feature: Being a modern imperative math programming language.

Therefore, instead of a compiler, TenLang will come with several different transpilers into existing ecosystems. Hereby, any frameworks built in TenLang will be usable in _any_ of those ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] C++ with Eigen
* [Future] Octave / MatLab
* [Future] R

## Roadmap

- [x] Project Skeleton
- [x] Number Primitives
- [x] String Primitives
- [x] Array Literals
- [x] Function Interfaces
- [x] Int call / subscript keying
- [x] Overloading & Call Type Checking
- [ ] Type Inheritance
- [ ] If / Else, Guard
- [ ] Generic Type Inferral
- [ ] Structs
- [ ] Interfaces
- [ ] Extensions
- [ ] Exceptions
- [ ] Staggered Dimensions
- [ ] Implicit tensor building
- [ ] Abstract functions + Higher order functions
- [ ] NDArrays
- [ ] Tuple Dimension Index
- [ ] Object Dimension Index ("Dictionaries"), Dictionary Literals
- [ ] Int Range Dimension Index
- [ ] Auto Broadcast
- [ ] Optionals
- [ ] String comprehension
- [ ] Sets
- [ ] Standard Library
- [ ] System Callback API / Permission Contexts

### Currently not planned

- Compiler / LLVM / Virtual Machine
  - See [Transpilation Targets](#Transpilation Targets)
- OOP / Polymorphism / Classes
  - Very complex and of limited use for most mathematical applications.
- Variadic parameters
  - Seldom used in practice. Passing array primitives is more versatile.
- for / for-each loops
  - .map / .forEach calls do the same. Instead, there will be a strong callable integration allowing for return / break / continue statements inside an anonymous closure. 
- Global (mutable) variables
  - Global variables are usually bad practice. Instead, TenLang encourages context objects.
- System I/O (input, GUI, filesystem, streams etc.)
  - Very complex. TenLang encourages building that part of the program in the target ecosystem.
- Multithreading
  - Very complex, and of limited use for (pure) mathematical applications.
