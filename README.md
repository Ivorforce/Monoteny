# Monoteny

Welcome to the mathemagical land of Monoteny! 

Monoteny is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in programming languages like Swift, Rust and Python.  

## Philosophy

### First-Class Multiple Dimension Monads

In Monoteny, Monads consist of dimensions referred to by a _dimension specifier_ and indexed by some _monadic type_. This would commonly be array shapes or optionals, but might also be a dictionary keyset or something else.

Every Monad is statically treated as its unit type. At compile time, operations on it are translated to appropriate mapping operations.

There are two exceptions: First, when two monad definitions collide, they are broadcast to each other using the dimension specifiers. Second, with the `@` syntax, the encapsulating monad can be referred to. 

### Shape Safety and Generics

Monoteny aims to guarantee shape, lookup and generally array operations safety.

There is a reasonable reason no other language has yet attempted this: Shape resolving can be as hard as executing the program itself. It seems impossible to devise a system that could possibly cover every use-case. Without one, the language quickly falls apart.

Luckily, we know how to solve hard problems in a readable and approachable way. It's programming.

Monoteny takes this to heart: Generic types are resolved with user code at compile time. The code is Monoteny, so it is unnecessary to learn a separate language paradigm.

### Subject Oriented Polymorphism

Polymorphism in Monoteny is offered using traits with abstract functions. When some trait conformance is required for a function, the function is polymorphically callable.

The implication depends on caller context. Specifically, it depends on whether the object has a deterministic polymorphism.

- Deterministic polymorphism (regular): Polymorphism can be resolved completely at compile time. No polymorphism exists after compilation.
- Indeterministic polymorphism (from I/O): Polymorphism cannot be resolved at compile time. Abstract function calls on these objects are resolved with virtual function tables.


## Roadmap

### Transpilation Targets

Monoteny lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things. Therefore, instead, Monoteny aims to focus on its most vital feature: Being a modern imperative math programming language.

This will be possible by Monoteny coming with several different transpilers. Hereby, any algorithms built in Monoteny will be usable in _any_ of those ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] C++ with Eigen
* [Future] Octave / MatLab
* [Future] R

Note: Transpilation of some features is quite difficult and cannot be achieved in human-readable fashion easily. For that reason, documentation keywords exist to specify how to export some things to specific languages. Everything not explicitly exported will not have readability as a high priority.


### Monoteny 0.1 (Toy Language Stage)

- [x] Project Skeleton
  - [x] Number, String, Array Literals
- [x] Functions
  - [x] Int-keyed parameters
  - [x] Single expression function definition syntax
  - [x] Overloading & Call Type Checking
  - [x] Static member functions
- [x] Binary operators: + - * / || && > < >= <= == != % **
  - [x] Unary operators: + - !
  - [x] "Conjunctive Pairs" comparison syntax, ex.: a > b >= c == d

### Monoteny 0.2 (Proof of Concept Stage)

- [x] Traits
  - [x] Requirements (`if Number<A> {}`)
  - [x] Inheritance (`if Number<#A> { trait OtherTrait<#A> }`)
  - [ ] Subtype Coercion (`A: B`, `declare SomeTrait if Self: B { fun f() }`, `a.f()  // a: A`)
  - [ ] Abstract Functions, Conformance Declarations
  - [ ] Stored Properties (for traits with associated Self)
    - [ ] Structs from traits (`SomeTrait(a: a, b: b)`) - only for non abstract
      - [ ] Anonymous Structs: `... -> (a: Int, b: Float) ... return (a: a, b: b)`
    - [ ] Delegation (`delegate some_property`) (delegates all properties' traits to this trait)
    - [ ] Properties conforming to property-like functions (automatically?)
      - [ ] Dynamic properties implemented as functions
    - [ ] Tuples (`tuple(x, y, z)` -> `trait<A> { let x: A, y: A, z: A }`)
    - [ ] Deconstruction assignment
- [ ] Non-linear linking (allow references to identifiers declared below)
- [ ] Comments (with transpilation)
- [x] Generics
  - [x] Reverse generic type checking (output types determined from inputs)
  - [ ] Generic constants (`let pi = 3.1415926`, inferred type: `$Float`)
  - [x] Anonymous generics (`#A`)
    - [x] ... with implicit trait conformance requirements (`$Number` -> `if $Number: Number {}`)
    - [ ] ... recursive (`$$Number: $Number`)
    - [x] ... in imperative code (coercing types)
- [x] Custom patterns with keywords (unary / binary operators)
  - [ ] Custom precedence steps (with associativity) 
  - [ ] Deconstruction for patterns (e.g. `fun a[data: A] :: ...` or `fun {left: A} + (a: A, b: B) :: ...`) 
- [ ] 'equivalence transformation' syntax: `((a + b) * c).any()` becomes `a + b .. * c .. .any()`
  - [ ] 'transformation assignment' syntax: `a .= + 5`; `b .= .union(c)`
- [ ] Expression Scopes (`let a = { ... yield b; };`)
- [ ] If / Else, if let
- [ ] Type Alias, aka `String = Character[Int...]` (defining functions on alias doesn't define them for the equal type)
  - [ ] Enums / Enum type inheritance (achieved through type alias)
- [ ] Monads
  - [ ] Tuple Dimension Index
  - [ ] Object Dimension Index ("Dictionaries"), Dictionary Literals
  - [ ] Open Int Range Dimension Index, array start / end handle syntax (>|, <|)
  - [ ] Auto Broadcast
  - [ ] Varargs: Int keying with infinite parameters (syntax: `a...: Type[0...]` for `print(a, b)` and `a...: Type[String]` for `print(a: a, b: b)`)
  - [ ] Optionals
  - [ ] Sets
  - [ ] Dict Literals
  - [ ] Iterators
  - [ ] Defaults (`a: $Float[Default]` for parameters to be omittable)

### Monoteny 1.0

- [ ] Exceptions (as monads)
  - [ ] Early return syntax
- [ ] IntX, FloatX (big int & float, which are structs)
- [ ] IntNative, FloatNative (platform-optimized int and float)
- [ ] Local functions and declarations
  - [ ] Anonymous functions
- [ ] String Comprehension
- [ ] Imports and Namespaces
- [ ] Non-Varargs subscripts and subscript overloads
- [ ] Indeterministic polymorphism
  - [ ] Virtual function tables
  - [ ] Polymorphic Enums (enums with attached objects)
    - [ ] Anonymous Enums (`-> A | B`) 
  - [ ] Higher Order Functions
    - [ ] Deep Function Currying

### Monoteny 2.0

- [ ] Staggered Dimensions
  - [ ] Implicit Tensors
- [ ] Post Link-Time Shape Tests
  - [ ] Array Dimension Index (i.e. 'anonymous enum')
  - [ ] Closed Int Range Dimension Index
- [ ] Computation Tree Folding (resolve static results post link-time)
- [ ] System I/O API / Permission Contexts
- [ ] Pointer Monad (-> shared object reference, mutable or immutable)

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
- for / for-each loops
  - .map / .forEach calls do the same. Instead, there will be a strong callable integration allowing for return / break / continue statements inside an anonymous closure. 
- Global (mutable) variables
  - Global variables are usually bad practice. Instead, Monoteny encourages context objects.
- Multithreading
  - Very complex, and of limited use for (pure) mathematical applications.
- Unsafe / Reflection
  - Because Monoteny is transpiled, this is not applicable to the language.
