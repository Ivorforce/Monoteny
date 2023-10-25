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
def {'$Real[Cartesian]}.to_spherical() -> $Real[Spherical] :: {
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

More code can be found in the [examples](./examples) directory.

## Targets

The following targets will be officially supported:

* [WIP] Python with NumPy
* [Future] Monoteny Dialect (e.g. to the reader-centric "Math" dialect, which uses math-y symbols)
* [Future] Octave / MatLab
* [Future] C++ with Eigen
* [Future] R
* [Future] Julia
* [Future] LaTeX Expressions

In addition, the transpilation API will allow 3rd parties to target more ecosystems as plugins.
