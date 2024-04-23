# Monoteny

#### NOTE: Monoteny is in its early development. The documents refer to features as if done, although the implementation does not reflect it. A language spec does not exist because the language design is highly exploratory. Feel free to browse nonetheless.

Welcome to the mathemagical land of Monoteny! 

Monoteny is a programming language intended for making libraries. It transpiles to many other programming languages and ecosystems so nobody needs to reinvent the wheel!

Monoteny specializes in reusability, runtime safety and readability. Why not check out [Monoteny 101](https://www.craft.me/s/dYSfJhYM9TAsne)?

## Philosophy

Let's quick-fire some language design decisions:

- **Many transpilation targets:** Writing your code in Monoteny ensures it can be used by everyone.
- **Simplicity:** Monoteny is all about *your* logic. You can keep it simple; the compiler will figure out how to make it fast.
- **Implicit Safety:** Code compiled from Monoteny cannot do anything bad to you except crash or freeze the program.
- **Flexible Runtime:** Monoteny bends to your will. Change grammar, decorate classes, generate code - it supports it all.
- **Monomorphization:** Compiled monoteny code doesn't use dynamic dispatch. This makes it fast and safe!
- **Monads:** Monoteny loves monads. Monads make your code short, easy to read, and fast!
- **Infinite Re-Usability:** All types are composable, inheritable and constructable. Define a concept once; use it forever!

## Code

```
-- Define structural named tuples.
tuple Cartesian(x, y, z);
tuple Spherical(l, e, a);

-- Define a function with a monadic input and a monadic output.
def (self '$Real[Cartesian]).to_spherical() -> $Real[Spherical] :: {
  -- Destructure to x, y, z arrays, each '$Real
  let #(x, y, z) = self;

  -- Pre-compute xz_sq
  let xz_sq = x ** 2 + z ** 2;

  -- Construct a monad $Real[Spherical] using a generic constructor.
  return #(
    l: (xz_sq + y ** 2).sqrt(),
    e: xz_sq.sqrt().atan2(y),
    a: z.atan2(x),
  );
};

def main! :: {
  -- Define dimensions
  let n, coord;
  
  -- Generate our input randomly, for demonstration's sake. Each entry gets a different random value due to broadcasting.
  let xyz 'Float32[n: 100, coord: Cartesian] = random();
  
  -- Call the function to create a multimonad 'Float32[n: 100, coord: Spherical] 
  let lea = xyz@.to_spherical()->[coord];
  
  -- Print the multimonad
  print(lea);
};

def transpile! :: {
  transpiler.add(main);
};
```

More code can be found in the [test-code](./test-code) (unit tests) and [monoteny](./monoteny) (standard library)  directories.

## A language with unique strengths and weaknesses

**Monoteny** makes a few decisions that are pretty unusual. Because it's not a general purpose language, it can double down on its design principles, and focus on making code readable, re-usable and safe. Anything you can't do in **Monoteny**, you can still hack together in the target environment, after all.


#### You'll like Monoteny for these tasks:


- **Create safe and reusable libraries**
    - Parse a **.yaml** file to a data structure.
    - Annotate all heartbeats in an ECG.
    - Read the metadata from a `.mp3` file.
    - Approximate the light spectrum hitting a solar panel at some geolocation.
    - Boost the available capabilities of other languages with versatile metaprogramming.
- **Solve deterministic problems**
    - Determine `pi` to the n'ths decimal.
    - Calculate the atmospheric pressure at the equator.
    - Create a graph for your expenses from a .CSV file.
    - Test your algorithm's floating point accuracy with perfect precision rationals.
- **Write code anyone can read**
  - Transpile as readable code for many languages.
  - Create LaTeX formulas from code.
  - Create pseudocode of known algorithms in customizable style.

#### Monoteny isn't well-suited for:

- Making application software (use Swift / C++ / Java instead).
- Making a web backend (use Typescript / Go / Erlang instead).
- Managing a database (use SQL instead).
- Driving a microchip (use C / zig / nim instead).
- Creating low level architecture (use Rust / C++ instead).


## Targets

The following languages are planned as compilation targets:

* [WIP] Python with NumPy
* [Future] Monoteny Dialect (e.g. to the reader-centric "Math" dialect, which uses math-y symbols)
* [Future] Octave / MatLab
* [Future] C++ with Eigen
* [Future] R
* [Future] Julia
* [Future] LaTeX Expressions

In addition, a transpilation API will allow 3rd parties to target custom ecosystems.

## How to run

The compiler is made with Rust. So first install Rust.
Then, you can use the following commands:

- `cargo build`: Build the project.
- `cargo test`: Run the unit tests.
- `cargo run`: Get the available commands for running. 
- `cargo run transpile -h`: Get info about the transpile subcommand. 
- `cargo run transpile --input test-code/hello_world.monoteny --all`: Transpile hello world to all currently available targets.

There is also a textmate grammar file for the language at [resources/Monoteny.tmbundle](./resources/Monoteny.tmbundle).
