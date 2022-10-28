# Monoteny

Welcome to the mathemagical land of Monoteny! 

Monoteny is an experimental language focusing on tensor math. It aims to streamline complex syntax, borrowing from modern trends in programming languages like Swift, Rust and Python.  

## Philosophy

### Ideologic Simplicity

Many languages adopt different concepts because they may map differently to machine code. Instead, Monoteny strives for minimal logic duplication: Many concepts can emerge naturally from properly abstracted concepts. Once one is recognized, it can still later be optimized by the compiler without losing semantic minimalilty.

This simplicity also allows for dialect agnosticity: Many language design choices are arbitrary and better offloaded to a dialect designer. Indeed, different groups of users may prefer different dialects to better understand the piece of logic. Monoteny encourages this to aid adoption. 

### First-Class Multiplicity

Monoteny recognizes multiplicity usually comes in predictable forms:

- Some unknown number of objects of the same type.
  - Monads (-> Arrays)
- Some known number of objects of different types.
  - Traits (-> Structs)
- One object of varying subtype.
  - Poly (-> Enums)

Monads are be treated as their unit. Monoteny allows multiple _dimensions_ in the same monad that auto-broadcast or match. The monadic wrapped must be explicitly referenced if needed (`a@[0]`).

Through subject-oriented design, objects are often coerced into simple traits. Hereby, complex algorithms can be used in different contexts without glue logic.

Polymorphic types are harder to work with, but are still supported because they are still needed for some use-cases. Some definitions may still be exposed from the original trait. 

### Functional and Impure

Programs are, usually, pure and deterministic. At compile-time they are folded to a minimal representation.

In other languages, 4 concepts usually prevent this type of folding:
- Global Mutables
  - Monoteny does not allow global mutables. Instead, parameters must be explicitly passed.
- Functional Impurities (e.g. I/O, Random...)
  - Monoteny requires explicitly impurity declarations.
    - `Float[Var]`: Values are unknown at design-time.
- Type Loss (e.g. multi-object array, polymorphism).
  - Monoteny uses statically typed multiplicity and subject-oriented function calls.

As a side effect to folding, often promises can be made about the code: For example, an array lookup into an empty array will always fail. Monoteny allows user code at fold-time to make promises about the code, from used types, parameters or impurities.

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
  - [ ] Generic values from literals (`let b '$Float = 5`)
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
- [x] Constant-Like function syntax (without `()`)
- [x] Generics
  - [x] Reverse generic type checking (output types determined from inputs)
  - [x] Implicit generics (`#A`)
    - [x] ... in imperative code
    - [x] ... with implicit trait conformance requirements (`$Number` -> `if $Number: Number {}`)
    - [ ] ... recursive (`$$Number: $Number`)
    - [ ] ... anonymous (`#(a: a, b: b)` or `#.a`)
- [x] Custom patterns with keywords (unary / binary operators)
  - [ ] Right-Unary Operators
  - [ ] Custom precedence steps (with associativity) 
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
- [ ] Meta Traits (traits whose instantiations can act as traits)
  - [ ] IntX, FloatX (variable bitcount int and float) - regular ints and floats are just 'optimized special cases' of this
- [ ] IntUnbound (int that can store any value)
- [ ] IntNative, FloatNative (platform-optimized int and float)
- [ ] Imports
- [ ] Local functions and declarations
  - [ ] Anonymous functions
- [ ] String Comprehension
- [ ] Imports and Namespaces
- [ ] Decorators (on structs, definitions)
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
- [ ] Optimization Definitions (e.g. an alternative implementation for sort() if all objects are ints)
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
