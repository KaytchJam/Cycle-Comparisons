mod utils;

use utils::cycles::ResetCounter;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

// CYCLE TEST THAT RECORDS EXECUTION TIME EACH DURING EACH ITERATION
fn run_inner_cycle_test(num_iters: usize, increment_amt: usize, reset_num: usize, counter_type: ResetCounter) -> Vec<u128> {
    let reset_func: Box<dyn Fn(usize, usize) -> usize> = counter_type.get_reset_func();
    let mut counter = 0_usize;
    let mut times: Vec<u128> = Vec::with_capacity(num_iters);

    // RUN OUR LOOP
    for _ in 0..num_iters {
        let start: Instant = Instant::now();
        counter += increment_amt;
        counter = reset_func(counter, reset_num);
        times.push(start.elapsed().as_nanos());
    }

    return times;
}

// CYCLE TEST THAT RECORDS TOTAL ITERATION TIME 
fn run_outer_cycle_test(num_outer_iters: usize, num_inner_iters: usize, increment_amt: usize, reset_num: usize, counter_type: ResetCounter) -> Vec<u128> {
    let reset_func: Box<dyn Fn(usize, usize) -> usize> = counter_type.get_reset_func();
    let mut counter = 0_usize;
    let mut times: Vec<u128> = Vec::with_capacity(num_outer_iters);

    // RUN OUR LOOP
    for _ in 0..num_inner_iters {
        let start = Instant::now();
        for _ in 0..num_inner_iters {
            counter += increment_amt;
            counter = reset_func(counter, reset_num);
        }
        times.push(start.elapsed().as_nanos());
    }

    return times;
}

/// RETURNS A VECTOR OF OUR INNER AND OUTER TESTS
fn run_cycle_tests(num_outer_iters: usize, num_inner_iters: usize, increment_amt: usize, reset_num: usize) -> (Vec<Vec<u128>>, Vec<Vec<u128>>) {
    let mut all_inner_times: Vec<Vec<u128>> = Vec::with_capacity(ResetCounter::cardinality());
    let mut all_outer_times: Vec<Vec<u128>> = Vec::with_capacity(ResetCounter::cardinality());

    // RUNNING OUR LOOPS ~ doing them seperately so I don't fuck with the cache locality too much
    // Inner Loop Test
    for reset_method in ResetCounter::into_iter() { 
        all_inner_times.push(run_inner_cycle_test(num_outer_iters, increment_amt, reset_num, reset_method));
    }
    // Outer Loop Test
    for reset_method in ResetCounter::into_iter() {
        all_outer_times.push(run_outer_cycle_test(num_outer_iters, num_inner_iters, increment_amt, reset_num, reset_method));
    }

    return (all_inner_times, all_outer_times);
}

/// EXPORTS OUR TIMES AS A CSV TO DIRECTORY `dir_path`
fn times_to_csv<T: ToString>(dir_path: &str, name: &str, data: &Vec<Vec<T>>) -> Result<(),std::io::Error> {
    let comma_func = |i: usize, lim: usize| if i == lim - 1 { "" } else { "," };
    let s1 = &ResetCounter::iter()
        .enumerate()
        .map(|(i,r)| (r.to_string().to_owned() + comma_func(i, ResetCounter::cardinality())))
        .collect::<Vec<String>>();

    let columns: Vec<&str> = [&["Iteration,"], s1.iter().map(|s: &String| s.as_str()).collect::<Vec<&str>>().as_slice()].concat();
    let mut f: File = File::create(dir_path.to_owned() + name)?;

    // WRITE COLUMNS
    for i in columns.iter() { f.write(i.as_bytes())?; } f.write("\n".as_bytes())?;
    // WRITE THE DATA
    for j in 0..data[0].len() {
        f.write(((j + 1).to_string() + ",").as_bytes())?;
        for i in 0..data.len() { 
            f.write((data[i][j].to_string() + comma_func(i, data.len())).as_bytes())?; 
        }
        f.write("\n".as_bytes())?;
    }

    Ok(())
}

fn main() {
    // CONSTANTS
    const TOTAL_OUTER_ITERS: usize = 5000;
    const TOTAL_INNER_ITERS: usize = 5000;
    const INCREMENT_AMOUNT: usize = 1;
    const RESET_NUMBER: usize = 10;
    // PATH CONSTANT(S)
    const OUTPUT_PATH: &str = "output/data/";

    // RUNNING OUR TESTS
    let (inner_times, outer_times) = run_cycle_tests(TOTAL_OUTER_ITERS, TOTAL_INNER_ITERS, INCREMENT_AMOUNT, RESET_NUMBER);
    times_to_csv(OUTPUT_PATH, "inner.csv", &inner_times).expect("Couldn't properly send inner data to a csv");
    times_to_csv(OUTPUT_PATH, "outer.csv", &outer_times).expect("Couldn't properly send outer data to csv");
}
