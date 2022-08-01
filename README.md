# TenLang

Welcome to the mathemagical land of TenLang! 

TenLang is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in functional and object-oriented languages. 

## Philosophy

### Implicit Object Multiplicity

In TenLang, every NDArray is treated as if it were an atom. At compile time, these operations are translated to appropriate iteration operations.

There are two exceptions: First, when two NDArrays collide, they are broadcast to each other appropriately. Second, with the `@[]` syntax, an object can be referred as an NDArray. 

### Shape Safety and Generics

TenLang aims to guarantee shape, lookup and generally array operations safety.

There is a reasonable reason no other language has yet attempted this: Shape resolving can be as hard as executing the program itself. It seems impossible to devise a system that could possibly cover every use-case. Without one, the language quickly falls apart.

Luckily, we know how to solve hard problems in a readable and approachable way. It's coding.

TenLang takes this to heart: Generic types are resolved with user code at compile time. The code itself follows TenLang syntax, so it is unnecessary to learn a separate complicated language. I hope this truly covers all (computable) use-cases. 

### Collections Combinations

In many languages, several independent types of collections exist, e.g. arrays, named tuples (-> 3d points, 2D size) and dictionaries. While 'Collection' interfaces support some number of functions, often algorithms end up being implemented many times. 

In actuality, the only way the aforementioned collections differ is indexing: Arrays use consecutive ints, named tuples use compile-time strings, dictionaries use unconsecutive hashables.

TenLang interprets this as an opportunity for abstraction: By allowing NDArray dimensions arbitrary indexing, one can cover all these use-cases in the same NDArray.

## Roadmap

### Transpilation Targets

TenLang lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things. Therefore, instead, TenLang aims to focus on its most vital feature: Being a modern imperative math programming language.

This will be possible by TenLang coming with several different transpilers. Hereby, any algorithms built in TenLang will be usable in _any_ of those ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] C++ with Eigen
* [Future] Octave / MatLab
* [Future] R

### TenLang 0.1

- [x] Project Skeleton
- [x] Number Primitives
- [x] String Primitives
- [x] Array Literals
- [x] Function Interfaces
- [x] Int call / subscript keying
- [x] Binary operators: + - * / || && > < >= <= == != % **
- [ ] Generic Type Inferral
- [ ] Overloading & Call Type Checking
- [ ] Interface Inheritance
- [ ] Unary operators: - !
- [ ] If / Else, Guard
- [ ] Multiple comparison syntax, ex.: a > b >= c == d (evaluated pairwise)
- [ ] Structs
- [ ] Interfaces + Extensions
- [ ] NDArrays
- [ ] 'equivalence transformation' syntax: (a == b).any() becomes a == b | .any()
- [ ] Tuple Dimension Index
- [ ] Object Dimension Index ("Dictionaries"), Dictionary Literals
- [ ] Open Int Range Dimension Index, array start / end handle syntax (>|, <|)
- [ ] Auto Broadcast
- [ ] Optionals
- [ ] Sets

### TenLang 1.0

- [ ] System Callback API / Permission Contexts
- [ ] Exceptions
- [ ] Abstract functions + Higher order functions
- [ ] Deep Function Currying
- [ ] String comprehension

### TenLang 2.0

- [ ] User-Defined Binary / Unary Operators (note: restricted to a set of characters like +^-=)
- [ ] Implicit tensor building
- [ ] Staggered Dimensions
- [ ] Array Dimension Index
- [ ] Closed Int Range Dimension Index

### Standard Library

- [ ] Standard unary + binary operators
- [ ] String Handling
- [ ] Common boolean resolving (all, any, none, etc.)
- [ ] Common functional dimension operations (filter, fold, etc.)
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
- Unsafe / Reflection
  - Because TenLang is transpiled, this is not applicable to the language.
