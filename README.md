# Monoteny

Welcome to the mathemagical land of Monoteny! 

Monoteny is an experimental language intended for making libraries and solving deterministic problems. It aims to expose logic using simple syntax, selectively borrowing concepts from many different languages that have worked well for this purpose.

## Philosophy

### Idealistic Simplicity

Many languages adopt concepts because they map to machine code in some desirable way. Instead, Monoteny strives for logic coherence: One piece of logic is reusable in many contexts, even if there are vastly different implications for the execution. In Monoteny, these implications are considered optimizations which are handled with hints and compiler logic.

On a syntactical level, many decisions are trade-offs between readability vs. brevity. Decisions often take into account one particular subset of domains, and may be arbitrary or obscure for other domains. Indeed, different groups of people may prefer different dialects to better understand some piece of logic. Monoteny supports and encourages dialects on a language level to aid adoption. 

### First-Class Multiplicity

Monoteny recognizes multiplicity (data) usually comes in predictable forms:

- Some known number of objects of different types.
  - Product Types (-> Traits / Structs)
- One object of varying subtype.
  - Tagged Union (-> Poly / Enums)
- Some unknown number of objects of the same type.
  - Monads (-> Arrays)

A monad is a wrapper that hides multiplicity on interaction with the type. Only when referenced (e.g. `a@[0]`) is interaction with the monadic wrapper permitted. A monadic wrapper might be anything from arrays over dictionaries to streams or nullability.

Meanwhile, objects come with a known type and can only be interacted with using declared traits. Functions cannot extract more knowledge from an object than they are given, aiding determinism. Often, objects are viewed from a minimalist perspective in order to reduce complexity.

Polymorphic types carry a sub-type per object. Generic traits defined on the supertype are adopted. For non-generically defined functions, the sub-types must be acted on one by one. 

### Functional and Impure

Large parts of programs can be designed in a pure and deterministic way. When compiling, Monoteny first generically unfolds function calls to resolve generic types, and then statically folds all constant code to minimize runtime cost.

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

Monoteny lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things. Therefore, instead, Monoteny aims to focus on its most vital feature: Being a minimal and understandable imperative logic language.

This will be possible by Monoteny coming with several different transpilers. Hereby, any algorithms built in Monoteny will be usable in _any_ of those ecosystems. The transpilation targets are:

* [WIP] Python with NumPy
* [Future] Monoteny Dialect
* [Future] Octave / MatLab
* [Future] C++ with Eigen
* [Future] R
* [Future] LaTeX (expressions)

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
  - [x] "Conjunctive Pairs" comparison syntax, ex.: a > b >= c

### Monoteny 0.2 (Proof of Concept Stage)

- [x] Generics
  - [x] Reverse generic type checking (output types determined from inputs)
  - [x] Implicit generics (`#A`)
    - [x] ... in imperative code
    - [x] ... with implicit trait conformance requirements (`$Number` -> `if $Number: Number {}`)
    - [ ] ... recursive (`$$Number: $Number`)
    - [ ] ... anonymous (`#(a: a, b: b)` or `#.a`)
- [x] `trait`: Objects that functions can be associated with.
  - [x] `trait` `inherit`: Require trait conformance to another trait
  - [ ] Stored Properties (for traits with associated Self)
    - [ ] Structs from traits (`SomeTrait(a: a, b: b)`) - only for non abstract
    - [ ] Anonymous Structs: `... -> (a: Int, b: Float) ... return (a: a, b: b)`
    - [ ] Delegation (`delegate some_property`) (delegates all properties' traits to this trait)
    - [ ] Properties conforming to property-like functions (automatically?)
      - [ ] Dynamic properties implemented as functions
    - [ ] Deconstruction assignment (`let (x, y, z) = vec`)
      - [x] `let`: Assign new variables
      - [x] `upd`: Change existing variables
      - [ ] `cnf`: Refutably assert equality to existing variables
  - [ ] Tuples (`tuple Vec3(x, y, z)`, of monadic type with struct-like initializer)
  - [ ] Subtype Coercion (`A: B`, `declare SomeTrait if Self: B { fun f() }`, `a.f()  // a: A`)
- [ ] Modules (imports)
  - [ ] Generic Unfolding: Compile functions with deeply resolved generics
  - [ ] `use` statements: Use parts of a module without changing or re-exporting it.
  - [ ] `abstract` functions: Declare functions only later.
  - [ ] `@transpile` decorators: Functions that are called when making a transpilation target. 
  - [ ] `@private` decorators: Functions or traits that can only be referenced using qualified syntax.
  - [ ] `inherit` statements: Use and expose another module within your module, allowing additions and overrides.
    - [ ] Partial inheritance: Use generic unfolding to use only the parts of a module /  trait that is actually needed.
  - [ ] Abstract Functions, Conformance Declarations
  - [ ] Namespaces
- [ ] Control Callbacks (e.g. `def if(expression 'Bool, onTrue 'fun, onFalse 'Fun[Option]) { ... }`))
  - [ ] If / Else
  - [ ] If let (refutable patterns)
  - [ ] Guard: call a closure with everything after the guard as a function (e.g. `guard if let` or `guard with`)
  - [ ] Expression Scopes (`let a = { ... yield b; };`)
- [ ] Type Alias, aka `String = Character[Int...]` (defining functions on alias doesn't define them for the equal type)
  - [ ] Enums / Enum type inheritance (achieved through type alias)
- [ ] Monads
  - [ ] Tuple Dimension Index
  - [ ] Object Dimension Index ("Dictionaries"), Dictionary Literals
  - [ ] Open Int Range Dimension Index, array start / end handle syntax (|>, <|)
  - [ ] Auto Broadcast
  - [ ] Varargs: Int keying with infinite parameters (syntax: `a...: Type[0...]` for `print(a, b)` and `a...: Type[String]` for `print(a: a, b: b)`)
  - [ ] Optionals
  - [ ] Sets
  - [ ] Dict Literals
  - [ ] Iterators
  - [ ] Defaults (`a: $Float[Default]` for parameters to be omittable)
- Syntax
  - [x] Constant-Like function syntax (without `()`)
  - [x] `(a:)` syntax: 'argument keyed by its variable name' for consistent function definitions, -calls and deconstructions
  - [ ] 'equivalence transformation' syntax: `((a + b) * c).any()` becomes `a + b .. * c .. .any()`
    - [ ] 'transformation assignment' syntax: `a .= + 5`; `b .= .union(c)`
  - [x] Custom expression patterns with keywords (unary / binary operators)
    - [x] Right-Unary Operators
    - [ ] Custom precedence steps (with associativity)
  - [x] Comments
    - [ ] Documentation

### Monoteny 1.0

- [ ] Exceptions (as monads)
  - [ ] Early return syntax
- [ ] Meta Traits (traits whose instantiations can act as traits)
  - [ ] IntX, FloatX (variable bitcount int and float) - regular ints and floats are just 'optimized special cases' of this
- [ ] IntUnbound (int that can store any value)
- [ ] IntNative, FloatNative (platform-optimized int and float)
- [ ] Switch / Match
- [ ] Local functions and declarations
  - [ ] Anonymous functions
- [ ] String Comprehension
- [ ] Custom Decorators (on structs, definitions)
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
- [ ] Constant Folding (resolve static results post link-time)
- [ ] Optimization Definitions (e.g. an alternative implementation for sort() if all objects are ints)
- [ ] System I/O API / Permission Contexts
- [ ] Pointer Monad (-> shared object reference, mutable or immutable)
- [ ] String-Indexing of members of structs (reflection)

### Standard Library

- [ ] 0D-Math operations (pow, sqrt, etc.)
- [ ] Array manipulation (filter, add, etc.)
- [ ] Collection reductions (sum, std, any, etc.)
- [ ] 1D timeseries data functions (filter, gaussian, running mean, etc.)
- [ ] String Handling (e.g. UTF8 (-> UInt8), replace, index_of)

### Common Dialects

- [ ] Common (writer-centric for monoteny programmers)
- [ ] Human (reader-centric for everyone, pseudo code inspired)

### Currently not planned

- Compiler / LLVM / Virtual Machine
  - See [Transpilation Targets](#Transpilation Targets)
- for / for-each loops
  - .map / .forEach calls do the same. Instead, there will be a strong callable integration allowing for return / break / continue statements inside an anonymous closure. 
- Global (mutable) variables
  - Global variables are usually bad practice. Instead, Monoteny encourages context objects.
- Multithreading
  - Very complex, and of limited use for (pure) mathematical applications.
- Unsafe
  - Unsafe operations go against the philosophy of Monoteny. Changes to the language must be offered via plugins.
