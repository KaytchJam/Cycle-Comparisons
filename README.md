
### Cycle Comparison

#### About.

This project is about comparing different types of "reset counters" in code, i.e. cases where you have some value incrementally increasing until it hits some reset value `reset`. At which point it resets back to a base value again. The following snippet below is an example of one:

```rust
fn if_example(loop_times: usize, counter: usize, reset: usize) {
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

**IfCounter**
- `IfCounter` is the case from before, where we use a conditional statement as our means of resetting our counter. 

**ModuloCounter**
- One of the earliest reset counters I was introduced to (aside from the if conditional) was the `ModuloCounter` which takes the form `counter = counter % reset`. It's easy to understand so long as you know what modulo does!

```rust
fn modulo_example(loop_times: usize, counter: usize, reset: usize) {
    for i in 0..loop_times {
        counter += 1;
        counter = counter % reset;
    }
}
```

**BooleanCounters**
- These methods take advantage of the binary returns of boolean operations, 0 or 1. In the case of `BooleanMulCounter` we make a logical comparison between our counter and our reset value, and then multiply negation of the value of that comparison with counter itself.

```rust
fn boolean_mul_example(loop_times: usize, counter: usize, reset: usize) {
    for i in 0..loop_times {
        counter += 1;
        counter = counter * !(counter == reset);
    }
}
```

- The `BooleanWrapCounter` follows the same logic, where we form a logical comparison between our counter and reset and negate the result. Where it differs it what we do after. Rather than multiply it with counter as we did before, we subtract it from 0 and perform a bitwise AND operation between counter and that value. What's taken advantage of here is the wrapping trait of integers. `0x00 - 0x01 = 0xFF` if we're working with 8 bit integers, `0x0000 - 0x0001 = 0xFFFF` under 16 bit, and so on. We can then treat it as a mask and AND it with counter. 

```rust
fn boolean_wrap_example(loop_times: usize, counter: usize, reset: usize) {
    for i in 0...loop_times {
        counter += 1;
        counter = counter & 0_usize.wrapping_sub(!(counter == reset));
    }
}
```

Time will be handled through `std::time::Instant` and we'll measure it in nanoseconds. All our metrics will be record as follows...

```rust
/// Records the time it takes for loop_times uses of ResetCounter counter_type
fn run_outer_cycle_test(num_outer_iters: usize, loop_times: usize, increment_amt: usize, reset: usize, counter_type: ResetCounter) -> Vec<u128> {
    let reset_func: Box<dyn Fn(usize, usize) -> usize> = counter_type.get_reset_func();
    let mut counter = 0_usize;
    let mut times: Vec<u128> = Vec::with_capacity(num_outer_iters);

    // We'll be getting NUM_INNER_ITERS samples of our loop runtime
    for _ in 0..num_inner_iters {
        let start = Instant::now();

        for _ in 0..loop_times {
            counter += increment_amt;
            counter = reset_func(counter, reset);
        }

        times.push(start.elapsed().as_nanos());
    }

    return times;
}

/// THE ABOVE IS RUN FOR EACH RESET_COUNTER
fn example() {
    for reset_method in ResetCounter::into_iter() {
        run_outer_cycle(num_outer_iters, loop_times, increment_amt, reset, reset_method);
    }
}
```

#### Todo

- [x] Generate Frequency Distributions for each Counter Method
- [ ] Observe whether changes in the 'RESET_NUMBER` influence performance
