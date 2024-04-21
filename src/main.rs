mod utils;

use utils::cycles::ResetCounter;
use core::num;
use std::fs::File;
use std::time::Instant;

// CYCLE TEST THAT RECORDS EXECUTION TIME EACH DURING EACH ITERATION
fn run_inner_cycle_test(num_iters: usize, increment_amt: usize, reset_num: usize, counter_type: ResetCounter) -> Vec<f64> {
    let reset_func: Box<dyn Fn(usize, usize) -> usize> = counter_type.get_reset_func();
    let mut counter = 0_usize;
    let mut times: Vec<f64> = Vec::with_capacity(num_iters);

    // RUN OUR LOOP
    for _ in 0..num_iters {
        let start = Instant::now();
        counter += increment_amt;
        counter = reset_func(counter, reset_num);
        times.push(start.elapsed().as_secs_f64());
    }

    return times;
}

// CYCLE TEST THAT RECORDS TOTAL ITERATION TIME 
fn run_outer_cycle_test(num_outer_iters: usize, num_inner_iters: usize, increment_amt: usize, reset_num: usize, counter_type: ResetCounter) -> Vec<f64> {
    let reset_func: Box<dyn Fn(usize, usize) -> usize> = counter_type.get_reset_func();
    let mut counter = 0_usize;
    let mut times: Vec<f64> = Vec::with_capacity(num_outer_iters);

    // RUN OUR LOOP
    for _ in 0..num_inner_iters {
        let start = Instant::now();
        for _ in 0..num_inner_iters {
            counter += increment_amt;
            counter = reset_func(counter, reset_num);
        }
        times.push(start.elapsed().as_secs_f64());
    }

    return times;
}

/// RETURNS A VECTOR OF OUR INNER AND OUTER TESTS
fn run_cycle_tests(num_outer_iters: usize, num_inner_iters: usize, increment_amt: usize, reset_num: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut all_inner_times: Vec<Vec<f64>> = Vec::with_capacity(ResetCounter::cardinality());
    let mut all_outer_times: Vec<Vec<f64>> = Vec::with_capacity(ResetCounter::cardinality());

    // RUNNING OUR LOOPS ~ doing them seperately so I don't fuck with the cache locality too much
    for reset_method in ResetCounter::into_iter() { 
        all_inner_times.push(run_inner_cycle_test(num_outer_iters, increment_amt, reset_num, reset_method));
    }

    for reset_method in ResetCounter::into_iter() {
        all_outer_times.push(run_outer_cycle_test(num_outer_iters, num_inner_iters, increment_amt, reset_num, reset_method));
    }

    return (all_inner_times, all_outer_times);
}

fn main() {
    // CONSTANTS
    const TOTAL_OUTER_ITERS: usize = 50;
    const TOTAL_INNER_ITERS: usize = 50;
    const INCREMENT_AMOUNT: usize = 1;
    const RESET_NUMBER: usize = 10;

    // RUNNING OUR TESTS
    let (inner_times, outer_times) = run_cycle_tests(TOTAL_OUTER_ITERS, TOTAL_INNER_ITERS, INCREMENT_AMOUNT, RESET_NUMBER);
}
