
### Cycle Comparison

#### About.

This project is about comparing different types of "reset counters" in code, i.e. cases where you have some value incrementally increasing until it hits some reset value `r`. At which point it resets back to a base value again. The following snippet below is an example of one:

```rust
fn example() {
    let loop_times: usize = 20;
    let reset: usize = 5;
    let mut counter: usize = 0;

    for i in 0..loop_times {
        counter += 1;
        // This is our reset function
        if counter >= reset { counter = 0; }
    }
}
```

Our value `reset`, initially set to 0, perpetually increments itself while within the `0..loop_times` for loop. Upon reaching the value `reset` its value is then set to 0 once again. The if block is our "ResetCounter" in this case, we'll call it an `IfCounter`.

#### Project Structure.

As stated before what we want to do is compare different "reset functions" in Rust and see how each of them perform relative to one another. We'll be using the following enum `ResetCounter` to represent each of our different types of reset counters and their respective functions.

```rust
enum ResetCounter {
    IfCounter = 0,
    ModuloCounter = 1,
    BooleanMulCounter = 2,
    BooleanWrapCounter = 3
}
```

Time will be handled through `std::time::Instant` and we'll measure it in nanoseconds.