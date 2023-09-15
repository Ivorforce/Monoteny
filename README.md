# Monoteny

Welcome to the mathemagical land of Monoteny! 

Monoteny is a programming language intended for making libraries. It transpiles to many other programming languages and ecosystems so you don't need to reinvent the wheel!

Monoteny specializes in reusability, runtime safety and readability. Why not check out [Monoteny 101](https://www.craft.me/s/dYSfJhYM9TAsne)?

## Philosophy

Let's quick-fire some language design decisions:

- **Many transpilation targets:** Writing your code in Monoteny ensures it lives forever.
- **Implicit Safety:** Monoteny code cannot do anything bad to you except crash or freeze the program. There are no syscalls!
- **Flexible Runtime:** You can generate new code from text, tokens or specialization on the fly.
- **Monomorphization:** Transpilation will result in code that doesn't use dynamic dispatch, even when using generics. This makes it fast!
- **Monads:** Monoteny loves monads. Monads make your code short and easy to read!
- **Vectorization:** Writing Monoteny code means writing vectorized code. This can be compiled to be _very_ fast.
- **Infinite Re-Usability:** All interface types are generic and thus replaceable. You can finally stop worrying about writing the same function _again_.

## Code

```
-- Define structural named tuples.
tuple Cartesian(x, y, z);
tuple Spherical(l, e, a);

-- Define a function with a monadic input and a monadic output.
def {'Float[Cartesian]}.to_spherical() -> Float[Spherical] :: {
  -- Destructure to x, y, z arrays, each 'Float
  let #(x, y, z) = self;

  -- Pre-compute xz_sq
  let xz_sq = x ** 2 + z ** 2;

  -- Construct a monad Float[Spherical] using a generic constructor.
  return #(
    l: (xz_sq + y ** 2).sqrt(),
    e: xz_sq.sqrt().atan2(y),
    a: z.atan2(x),
  );
};

@main
def main() :: {
  -- Define dimensions
  let n, coord;
  
  -- Generate our input randomly, for demonstration's sake. Each entry gets a different random value due to broadcasting.
  let xyz 'Float32[n: 100, coord: Cartesian] = random();
  
  -- Call the function to create a multimonad 'Float32[n: 100, coord: Spherical] 
  let lea = xyz@.to_spherical()->[coord];
  
  -- Print the multimonad
  print(lea);
};

@transpile
def transpile(transpiler 'Transpiler) :: {
  transpiler.add(main);
};
```

More code can be found in the [examples](./examples) directory.

## Roadmap

### Targets

Monoteny lacks many features required to build full apps. Luckily, many excellent ecosystems exist where it is possible to build such things.
Therefore, instead, Monoteny aims to focus on its most vital feature: Being a minimal and understandable imperative logic language.

This will be possible through Monoteny coming with several transpilers.
Hereby, any algorithms built in Monoteny will be usable in _any_ of those ecosystems.
In addition, an interpreter is used to fold constant code and run during transpile time.
The planned targets are:

* [WIP] Interpreter
* [WIP] Python with NumPy
* [Future] Monoteny Dialect
* [Future] Octave / MatLab
* [Future] C++ with Eigen
* [Future] R
* [Future] LaTeX (expressions)

Note: Monoteny has some features that do not translate easily to other languages. For example: In Monoteny, a constant is a function requiring arguments on the return type and inferred function binds.
This amount of flexibility would be confusing and difficult to use in Python or C.

For this reason, transpilation responsibility is handed over to the programmer.
They will ultimately decide how, especially w.r.t. different languages, code should be transpiled.
What this results in is a somewhat unusual 2-layer transpilation: Those functions and types that are designed to work with the outside, and those that 'just need to work'.


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
  - [x] Monomorphization: Specialize functions and types at compile-time.
    - [ ] Generic Export: Allow the export of unspecialized functions through a trait conformance parameter. 
  - [x] Reverse generic type checking (output types determined from inputs)
  - [x] Implicit generics (`#A`)
    - [x] ... in imperative code
    - [x] ... with implicit trait conformance requirements (`$Number` -> `if $Number: Number {}`)
    - [ ] ... recursive (`$$Number: $Number`)
    - [ ] ... anonymous (`#(a: a, b: b)` or `#.a`)
- [x] `trait`: Objects that functions can be associated with.
  - [x] `trait` `inherit`: Require trait conformance to another trait
  - [ ] Stored Properties (for traits with associated Self)
    - [ ] Structs from traits (`SomeTrait(a: a, b: b)`) - only for non abstract traits
    - [ ] Anonymous Structs: `... -> (a: Int, b: Float) ... return (a: a, b: b)`
    - [ ] Delegation (`delegate some_property`) (delegates all properties' traits to this trait)
    - [ ] Properties conforming to property-like functions (automatically?)
      - [ ] Dynamic properties implemented as functions
    - [ ] Deconstruction assignment (`let (x, y, z) = vec`)
      - [x] `let`: Assign new variables
      - [x] `upd`: Change existing variables
      - [ ] `cnf`: Refutably assert equality to existing variables
    - [ ] Generic: Any used generics will automatically generify the object
  - [ ] Tuples (`tuple Vec3(x, y, z)`, of monadic type with struct-like initializer)
  - [ ] Subtype Coercion (`A: B`, `declare SomeTrait if Self: B { fun f() }`, `a.f()  // a: A`)
- [ ] Modules (imports)
  - [ ] `use` statements: Use parts of a module without changing or re-exporting it.
  - [x] `@transpile` decorators: Functions that are called when making a transpilation target. 
  - [ ] `@private` decorators: Functions or traits that can only be referenced using qualified syntax.
  - [ ] `inherit` statements: Use and expose another module within your module, allowing additions and overrides.
  - [x] Abstract Functions, Conformance Declarations
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
  - [x] String Interpolation
- [x] Inlining trivial calls

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

- [x] 0D-Math operations (pow, sqrt, etc.)
- [ ] Array manipulation (filter, add, etc.)
- [ ] Collection reductions (sum, std, any, etc.)
- [ ] 1D timeseries data functions (filter, gaussian, running mean, etc.)
- [ ] String Handling (e.g. UTF8 (-> UInt8), replace, index_of)

### Common Dialects

- [ ] Common (writer-centric for monoteny programmers)
- [ ] Human (reader-centric for everyone, pseudo code inspired)

### Currently not planned

- Compiler / LLVM
  - See [Targets](#Targets)
- for / for-each loops
  - .map / .forEach calls do the same. Instead, there will be a strong callable integration allowing for return / break / continue statements inside an anonymous closure. 
- Global (mutable) variables
  - Global variables are usually bad practice. Instead, Monoteny encourages context objects.
- Multithreading
  - Very complex, and of limited use for (pure) mathematical applications.
- Unsafe
  - Unsafe operations go against the philosophy of Monoteny. Changes to the language must be offered via plugins.
