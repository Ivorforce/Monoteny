# TenLang

Welcome to the mathemagical land of TenLang! 

TenLang is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in programming languages like Swift, Rust and Python.  

## Philosophy

### First-Class Multiple Dimension Monads

In TenLang, Monads consist of dimensions referred to by a _dimension specifier_ and indexed by some _monadic type_. This would commonly be array shapes or optionals, but might also be a dictionary keyset or something else.

Every Monad is statically treated as its unit type. At compile time, operations on it are translated to appropriate mapping operations.

There are two exceptions: First, when two monad definitions collide, they are broadcast to each other using the dimension specifiers. Second, with the `@` syntax, the encapsulating monad can be referred to. 

### Shape Safety and Generics

TenLang aims to guarantee shape, lookup and generally array operations safety.

There is a reasonable reason no other language has yet attempted this: Shape resolving can be as hard as executing the program itself. It seems impossible to devise a system that could possibly cover every use-case. Without one, the language quickly falls apart.

Luckily, we know how to solve hard problems in a readable and approachable way. It's programming.

TenLang takes this to heart: Generic types are resolved with user code at compile time. The code is TenLang, so it is unnecessary to learn a separate language paradigm.

## Roadmap

### Transpilation Targets

TenLang lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things. Therefore, instead, TenLang aims to focus on its most vital feature: Being a modern imperative math programming language.

This will be possible by TenLang coming with several different transpilers. Hereby, any algorithms built in TenLang will be usable in _any_ of those ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] C++ with Eigen
* [Future] Octave / MatLab
* [Future] R

### TenLang 0.1 (Toy Language Stage)

- [x] Project Skeleton
  - [x] Number, String, Array Primitives
- [x] Functions
  - [x] Int-keyed parameters
  - [x] Single expression function definition syntax
  - [x] Overloading & Call Type Checking
  - [x] Static member functions
- [x] Binary operators: + - * / || && > < >= <= == != % **
  - [x] Unary operators: + - !
  - [x] "Conjunctive Pairs" comparison syntax, ex.: a > b >= c == d
  - [x] User-Defined Unary / Binary Operators

### TenLang 0.2 (Proof of Concept Stage)

- [ ] Forward generic type checking
- [ ] Comments (with transpilation)
- [ ] Subscript function syntax
- [ ] 'equivalence transformation' syntax: ((a + b) * c).any() becomes a + b .. * c .. .any()
  - [ ] 'transformation assignment' syntax: a .= + 5; b .= .union(c)
- [ ] Var-Like 0 parameter function syntax (let c = a.b; a.b = c;)
- [ ] Traits, x trait inheritances, trait abstract functions
- [ ] Expression Scopes (let a = { ... yield b; })
- [ ] If / Else, if let, Guard, Guard let
- [ ] Varargs: Int keying with infinite parameters (syntax: a...: Type[0...] for print(a, b) and a...: Type[String] for print(a: a, b: b))
- [ ] Structs
- [ ] Enums / Enum type inheritance
- [ ] Specializations: Raw data is some type, but additional functions will match
- [ ] Monads
  - [ ] Tuple Dimension Index
  - [ ] Object Dimension Index ("Dictionaries"), Dictionary Literals
  - [ ] Open Int Range Dimension Index, array start / end handle syntax (>|, <|)
  - [ ] Auto Broadcast
- [ ] Optionals
- [ ] Sets

### TenLang 1.0

- [ ] Reverse generic type checking (output types determined from inputs)
- [ ] System Callback API / Permission Contexts
- [ ] Exceptions
- [ ] Polymorphic Enums (attached objects)
- [ ] Abstract functions + Higher order functions
- [ ] Deep Function Currying
- [ ] String comprehension
- [ ] Non-Varargs subscripts and subscript overloads

### TenLang 2.0

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
- Runtime Polymorphism / Classes
  - Very complex and of limited use for most mathematical applications.
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
