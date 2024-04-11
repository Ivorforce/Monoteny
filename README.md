# Monoteny

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

More code can be found in the [test-code](./test-code) directory.

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
