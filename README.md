# TenLang

Welcome to the mathemagical land of TenLang! 

TenLang is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in functional and object-oriented languages. Here are some of the main philosophies:

- Every object and object collection is a broadcastable NDArray.
  - That includes arrays, sets, dictionaries and tuples.
- The compiler guarantees NDArray broadcast and lookup safety.
- All actions on an NDArray broadcast to their atoms, unless specified.
  - Yes, that includes function calls and member lookups.

## Roadmap

### Transpilation Targets

TenLang lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things. Therefore, instead, TenLang aims to focus on its most vital feature: Being a modern imperative math programming language.

This will be possible by TenLang coming with several different transpilers. Hereby, any algorithms built in TenLang will be usable in _any_ of those ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] C++ with Eigen
* [Future] Octave / MatLab
* [Future] R

### Language Features

- [x] Project Skeleton
- [x] Number Primitives
- [x] String Primitives
- [x] Array Literals
- [x] Function Interfaces
- [x] Int call / subscript keying
- [x] Binary operators: + - * / || && > < >= <= == != % **
- [ ] Unary operators: - !
- [ ] Overloading & Call Type Checking
- [ ] Type Inheritance
- [ ] If / Else, Guard
- [ ] Generic Type Inferral
- [ ] Multiple comparison syntax: a > b > c and a == b == c
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
- [ ] User-Defined Binary / Unary Operators (note: restricted to a set of characters like +^-=)
- [ ] System Callback API / Permission Contexts

### Standard Library

- [ ] Standard unary + binary operators
- [ ] String Handling
- [ ] Common 0D-Math operations (pow, sqrt, etc.)
- [ ] Common 1D-Math operations (sum, std, etc.)
- [ ] Common timeseries data functions (filter, gaussian, running mean, etc.)

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
