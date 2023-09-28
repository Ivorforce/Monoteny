# Monoteny

Welcome to the mathemagical land of Monoteny! 

Monoteny is a programming language intended for making libraries. It transpiles to many other programming languages and ecosystems so nobody needs to reinvent the wheel!

Monoteny specializes in reusability, runtime safety and readability. Why not check out [Monoteny 101](https://www.craft.me/s/dYSfJhYM9TAsne)?

## Philosophy

Let's quick-fire some language design decisions:

- **Many transpilation targets:** Writing your code in Monoteny ensures it can be used by everyone.
- **Simplicity:** Many good general purpose programming languages already exist. Monoteny aims to double down on its strengths, rather than supporting use-cases it's not well suited for.
- **Implicit Safety:** Monoteny code cannot do anything bad to you except crash or freeze the program. There are no (general purpose) syscalls!
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

def @main :: {
  -- Define dimensions
  let n, coord;
  
  -- Generate our input randomly, for demonstration's sake. Each entry gets a different random value due to broadcasting.
  let xyz 'Float32[n: 100, coord: Cartesian] = random();
  
  -- Call the function to create a multimonad 'Float32[n: 100, coord: Spherical] 
  let lea = xyz@.to_spherical()->[coord];
  
  -- Print the multimonad
  print(lea);
};

def @transpile :: {
  transpiler.add(main);
};
```

More code can be found in the [examples](./examples) directory.

## Roadmap

### Targets

The following targets are currently planned for active support:

* [WIP] Python with NumPy
* [Future] Monoteny Dialect
* [Future] Octave / MatLab
* [Future] C++ with Eigen
* [Future] R
* [Future] Julia
* [Future] LaTeX Expressions

Other targets may yet be implemented by other people.


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
  - [x] Reverse generic type checking (output types determined from inputs)
  - [x] Implicit generics (`#A`)
    - [x] ... in imperative code
    - [x] ... with implicit trait conformance requirements (`$Number` -> `if $Number: Number {}`)
    - [ ] ... recursive (`$$Number: $Number`)
    - [ ] ... anonymous (e.g. `#(a: a, b: b)` or `#.a`)
- [x] `trait`: Objects that functions can be associated with.
  - [x] `trait` `inherit`: Require trait conformance to another trait
  - [x] Abstract Functions, Conformance Declarations
  - [ ] Stored Properties (for traits with associated Self)
    - [ ] Structs from traits (`SomeTrait(a: a, b: b)`) - only for non abstract traits
    - [ ] Anonymous Structs: `... -> (a: Int, b: Float) ... return (a: a, b: b)`
    - [ ] Delegation (`@delegate(Eq) var ...`) (implement all / selected trait offered by the property by calling it on the property)
    - [ ] Deconstruction assignment (`let (x, y, z) = vec`)
      - [x] `let`: Assign new variables
      - [x] `upd`: Change existing variables
    - [ ] Generic: Any used generics will automatically generify the object
  - [ ] Tuples (`tuple Vec3(x, y, z)`, of monadic type with struct-like initializer)
- Running
  - [x] `@main` decorates functions that can be run from the cli.
  - [x] `@transpile` decorates functions that can transpile the code.
  - [ ] Pass parameters to `@main` and `@transpile` from the cli (array of strings).
- [ ] Modules (imports)
  - [ ] Namespaces
  - [ ] `use` statements: Use parts of a module without changing or re-exporting it.
  - [ ] `@private` decorators: Functions or traits that can only be referenced using qualified syntax.
  - [ ] `inherit` statements: Use and expose another module within your module, allowing additions and overrides.
- [ ] Control Callbacks (e.g. `def if(expression 'Bool, onTrue 'fun, onFalse 'Fun[Option]) { ... }`))
  - [ ] `if ... :: { } else :: { }`
  - [ ] `guard ... else :: { }`
- [x] Anonymous Blocks
  - [ ] Yield Statements (`let a = { ... yield b; };`)
- [ ] Type Alias, aka `String = Character[Int...]` (defining functions on alias doesn't define them for the equal type)
  - [ ] Enums / Enum type inheritance (achieved through type alias)
- [ ] Monads
  - [ ] Tuple Dimension Index
  - [ ] Defaults (`a: $Float[Default]` for parameters to be omittable)
  - [ ] Object Dimension Index ("Dictionaries"), Dictionary Literals
  - [ ] Open Int Range Dimension Index, array start / end handle syntax (|>, <|)
  - [ ] Auto Broadcast
  - [ ] Optionals
  - [ ] Sets
  - [ ] Iterators
- Syntax
  - [x] Constant-Like function syntax (without `()`)
  - [x] `(a:)` syntax: 'argument keyed by its variable name' for consistent function definitions, -calls and deconstructions
  - [ ] 'transformation assignment' syntax: `a .= $0 + 5`; `b .= $0.union(c)`
  - [x] Custom expression patterns with keywords (unary / binary operators)
    - [x] Right-Unary Operators
  - [x] Comments
  - [x] String Interpolation
  - Style transpilation
    - [ ] Comment & Documentation
    - [ ] Newline Separator transpilation
- [x] Simple Constant Folding
  - [x] Inline trivial calls (calls that are at most one call)
  - [ ] Auto-Delete objects without variables (e.g. for Console.write_line())

### Monoteny 1.0

- [ ] Exceptions (as monads)
  - [ ] Early return syntax
  - [ ] Refutable Patterns
    - [ ] `cnf`: Refutably assert equality to existing variables
    - [ ] `if let Some(a) = a :: { }`
    - [ ] `guard let Some(a) = a else { }`
- [ ] Meta Traits (traits whose instantiations can act as traits)
  - [ ] IntX, FloatX (types implementing int and float math depending on a bit count) 
    - [ ] Demote existing fixed-width ints and floats (e.g. Int32) to optimizations of IntX
- [ ] BigInt ($Int of auto-adjusting width)
- [ ] Generic Export: Allow the export of unspecialized functions through a trait conformance parameter.
- [ ] `match x with [0: { ... }]`
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
- [ ] Constant Folding (resolve static results post link-time)
- [ ] Varargs: Int keying with infinite parameters (syntax: `a...: Type[0...]` for `print(a, b)` and `a...: Type[String]` for `print(a: a, b: b)`)

### Monoteny 2.0

- [ ] Staggered Dimensions
  - [ ] Implicit Tensors
- [ ] Link-Time Interpreter
  - [ ] Computational (return-) types
  - [ ] Automatic shape tests for dimensions
    - [ ] Array Dimension Index (i.e. 'anonymous enum')
    - [ ] Closed Int Range Dimension Index
- [ ] Custom precedence steps (with associativity)
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
- [ ] CLI Argument Parser

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
