mod utils;

// plotter
use plotters::prelude::*;
use plotters::style::full_palette::{BLUE, BROWN, GREY};
// local
use utils::cycles::ResetCounter;
// std
use std::fs::File;
use std::io::BufReader;
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
    for _ in 0..num_outer_iters {
        let start = Instant::now();
        for _ in 0..num_inner_iters {
            counter += increment_amt;
            counter = reset_func(counter, reset_num);
        }
        times.push(start.elapsed().as_nanos());
    }

    return times;
}

fn outer_cycle_sample(num_outer_iters: usize, num_inner_iters: usize, increment_amt: usize, reset_num: usize, counter_type: ResetCounter) -> f32 {
    let reset_func: Box<dyn Fn(usize, usize) -> usize> = counter_type.get_reset_func();
    let mut counter = 0_usize;
    let mut time_sum = 0;

    for _ in 0..num_outer_iters {
        let start = Instant::now();
        for _ in  0..num_inner_iters {
            counter += increment_amt;
            counter = reset_func(counter, reset_num);
        }
        time_sum += start.elapsed().as_nanos();
    }

    return time_sum as f32 / num_outer_iters as f32;
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

// CHECK IF THE RELEASE FLAG IS SET
fn release_flag_prefix() -> &'static str {
    #[cfg(debug_assertions)]
    {
        return "debug_"
    }
    #[cfg(not(debug_assertions))]
    {
        return "release_";
    }
}

// Retrieves the maximum value in a vector of vectors
fn max_2d_vec(dist: &Vec<Vec<f32>>) -> u32 {
    let mut true_max: u32 = 0;
    for v in dist.iter() {
        true_max = true_max.max(v.iter().map(|e| (*e) as u32).max().unwrap());
    }
    return true_max;
}

fn plot_reset_counter_graphs(graph_path: &str, file_name: &str, dist: &Vec<Vec<f32>>, colors: &[plotters::style::RGBColor]) {
    let fname = graph_path.to_owned() + file_name + ".png";
    let drawing_area = BitMapBackend::new(&fname, (1280, 720))
        .into_drawing_area();

    drawing_area.fill(&WHITE).unwrap();
    let maximum: u32 = max_2d_vec(dist);
    let dist_max = maximum;

    let mut chart = ChartBuilder::on(&drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Reset Counter Times (ns)", ("san-serif", 36))
        .build_cartesian_2d(0..dist[0].len(), 0..dist_max)
        .unwrap();
    
    for idx in 0..ResetCounter::cardinality() {
        let method_color = &colors[idx];
        chart.draw_series(
            //dist[idx].iter().enumerate().map(|(x,y)| TriangleMarker::new((x,(*y) as u32), 5, method_color)
            LineSeries::new(dist[idx].iter().enumerate().map(|(x,v)| (x,*v as u32)), method_color)
        ).unwrap()
          .label(ResetCounter::from_index(idx).to_string())
          .legend(|(x,y)| Rectangle::new([(x - 15, y + 1), (x, y)], *method_color));
    }

    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(20)
        .legend_area_size(5)
        .border_style(BLUE)
        .background_style(BLUE.mix(0.1))
        .label_font(("Calibri", 20))
        .draw()
        .unwrap();

    chart.configure_mesh().draw().unwrap();
}

#[derive(Debug, Clone)]
struct Bin(std::ops::Range<usize>, usize);

impl Bin {
    fn new(min: usize, max: usize) -> Bin {
        return Bin(min..max, 0);
    }

    fn contains(&mut self, elem: usize) -> bool {
        return self.0.contains(&elem);
    }

    fn insert_or(&mut self, elem: usize) -> Result<&mut Self, &'static str> {
        if !self.contains(elem) {
            return Err("Not contained in this bin's range")
        }

        return Ok(self);
    }
 }

/// Constructs num_bins number of uniformly sized ranges and fills them based on matches with dist
fn construct_bin_vec(min: usize, max: usize, num_bins: usize, dist: &Vec<f32>) -> Vec<Bin> {
    let bin_sizes = ((max - min) as f32 / num_bins as f32).ceil() as usize;
    let mut hist: Vec<Bin> = Vec::with_capacity(num_bins);

    // CONSTRUCT BIN VECTOR
    let mut last_bin: usize = min;
    for _ in 0..num_bins {
        let new_last = last_bin + bin_sizes;
        hist.push(Bin::new(last_bin, new_last));
        last_bin = new_last;
    }

    // FILL BIN VECTOR
    for value in dist {
        let value = *value as usize;
        let mut last = hist.iter_mut().skip_while(|bin| bin.0.end < value).next();

        if last.is_some() {
            last.as_mut().unwrap().1 += 1;
        }
    }

    return hist;
}

// Retreive the bin with the maximum frequency
fn find_max_frequency(bin_vec: &Vec<Bin>) -> usize {
    let mut max_freq = 0;
    for bin in bin_vec {
        max_freq = max_freq.max(bin.1);
    }

    return max_freq;
}

fn dist_mean(dist: &Vec<f32>) -> f32 {
    return dist.iter().sum::<f32>() / dist.len() as f32;
}

fn dist_stdev(dist: &Vec<f32>, mean: f32) -> f32 {
    return dist.iter().map(|v| (*v - mean).powi(2)).sum::<f32>().sqrt() / (dist.len() - 1) as f32;
}

/// Constructs a bin vec with bins sized according to the mean and standard deviation passed in,
/// ranges outside of 3 stdevs shared 1 large unevenly sized bin
fn normal_dist(mean: f32, stdev: f32) -> Vec<Bin> {
    let mut bin_vec: Vec<Bin> = Vec::new();
    let z_to_x = |x: i32| (x as f32 * stdev + mean) as usize;

    bin_vec.push(Bin::new(usize::MIN, z_to_x(-3)));
    for i in -3..3 {
        bin_vec.push(Bin::new(z_to_x(i), z_to_x(i + 1)));
    }
    bin_vec.push(Bin::new(z_to_x(3), usize::MAX));
    return bin_vec;
}

fn fill_bin_vec<U: From<usize> + Into<usize>, T: IntoIterator<Item = U>>(bin_vec: &mut Vec<Bin>, iter: T) {
    for value in iter {
        let value: usize = value.into();
        let mut last = bin_vec.iter_mut().skip_while(|bin| bin.0.end < value).next();

        if last.is_some() {
            last.as_mut().unwrap().1 += 1;
        }
    }
}

// Search the vector of bins for its first nonzero value
fn first_nonzero_bin(bin_vec: &Vec<Bin>) -> usize {
    let nonzero = bin_vec.iter().skip_while(|bin| bin.1 == 0).next();
    return if nonzero.is_some() { nonzero.unwrap().0.start } else { 0 };
}

// Find the mean of the passed in Vector of Bins
fn mean(bin_vec: &Vec<Bin>) -> f32 {
    let mut weighted_sum = 0_f32;
    let mut total_inputs = 0;

    for bin in bin_vec.iter() {
        weighted_sum += bin.1 as f32 * ((bin.0.end - bin.0.start) / 2) as f32;
        total_inputs += bin.1;
    }

    return weighted_sum / total_inputs as f32; // placeholder
}

/// Generates a bar chart given a reference to a Vec<Bin> and exports it to \<dir\>
fn export_bar_chart(dir: &str, fname: &str, bin_vec: &Vec<Bin>, color: plotters::style::RGBColor) -> () {
    let file_path = dir.to_owned() + fname + ".png";
    let title = fname.to_owned() + " " + " Performance Distribution (nanoseconds)";
    let bmap = BitMapBackend::new(&file_path, (1280, 720))
        .into_drawing_area();

    bmap.fill(&WHITE).unwrap();
    let bin_max = bin_vec[bin_vec.len() - 1].0.end;
    let freq_max = find_max_frequency(bin_vec);
    let start_bin = first_nonzero_bin(bin_vec);
    
    let mut chart = ChartBuilder::on(&bmap)
        .set_label_area_size(LabelAreaPosition::Bottom, 16)
        .set_label_area_size(LabelAreaPosition::Left, 16)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(20)
        .caption(title, ("Calibri", 20))
        .build_cartesian_2d(start_bin..bin_max, 0..(freq_max + 100))
        .unwrap();

    chart.draw_series(
        bin_vec.iter().map(|b| {
            let mut bar = Rectangle::new([(b.0.start, 0), (b.0.end, b.1)], color.filled());
            bar.set_margin(0, 0, 5, 5);
            bar
        })
    ).unwrap();

    chart
        .configure_mesh()
        .y_desc("Count")
        .x_desc("Time (nanoseconds)")
        .draw().unwrap();
}

/// Generate a sampling distribution for the metric
fn gen_metric_sample_dist(num_samples: usize, outer_iters: usize, inner_iters: usize, reset_num: usize, counter: ResetCounter) -> Vec<f32> {
    let mut sample_dist: Vec<f32> = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        sample_dist.push(outer_cycle_sample(outer_iters, inner_iters, 1, reset_num, counter));
    }
    return sample_dist;
}

/// num_samples samples of outer test expected values
fn gen_all_outer_samples(num_samples: usize, outer_iters: usize, inner_iters: usize, reset_num: usize) -> Vec<Vec<f32>> {
    let mut sampling_dists: Vec<Vec<f32>> = Vec::new();
    for counter in ResetCounter::into_iter() {
        sampling_dists.push(gen_metric_sample_dist(num_samples, outer_iters, inner_iters, reset_num, counter));
    }
    return sampling_dists;
}

// checks if all bins in the bin_vec are empty
fn is_binvec_empty(bin_vec: &Vec<Bin>) -> bool {
    for bin in bin_vec.iter() {
        if bin.1 != 0 { return false; }
    }
    return true;
}

fn get_times_from_csv(fpath: &str) -> Result<Vec<Vec<f32>>,std::io::Error> {
    let all_times: Vec<Vec<f32>> = Vec::with_capacity(ResetCounter::cardinality());
    let file = File::open(fpath)?;
    let mut buf_reader = BufReader::new(file);
    return Result::Ok(all_times);
}

fn main() {
    // CONSTANTS
    const OUTPUT_PATH: &str = "output/data/";
    const GRAPH_PATH: &str = "output/graphs/";
    const PRESERVE_PATH: &str = "output/preserve/";
    const PRESERVE_FNAME: &str = "outer_averages.csv";
    const TOTAL_OUTER_ITERS: usize = 100;
    const TOTAL_INNER_ITERS: usize = 100;
    const NUM_SAMPLE_DISTS: usize = 1_000_000;
    const RESET_NUMBER: usize = 1;
    const PLOT_COLORS: [plotters::style::RGBColor; 5] = [RED, BLUE, BROWN, GREEN, GREY];

    // FOR BIN CONSTRUCTION
    const Z_SCORE: f32 = 100_f32;

    // EXECUTION MODE CHECK + GETTING FILE NAMES
    let fname_prefix = release_flag_prefix();
    let graph_out = GRAPH_PATH.to_owned() + &fname_prefix[0..(fname_prefix.len() - 1)] + "/";
    let preserve_in = PRESERVE_PATH.to_owned() + fname_prefix + "/" + PRESERVE_FNAME;

    // RUNNING OUR TESTS
    let dists: Vec<Vec<f32>> = gen_all_outer_samples(NUM_SAMPLE_DISTS, TOTAL_OUTER_ITERS, TOTAL_INNER_ITERS, RESET_NUMBER);
    times_to_csv(OUTPUT_PATH, (fname_prefix.to_owned() + "outer_averages.csv").as_str(), &dists)
        .expect("Couldn't properly send sampling distributions to csv");

    // PLOTTING
    plot_reset_counter_graphs(GRAPH_PATH, (fname_prefix.to_owned() + "counter_time_averages").as_str(), &dists, &PLOT_COLORS);
    let mut stats: Vec<f32> = Vec::with_capacity(ResetCounter::cardinality() * 2);

    // CONSTRUCTING BIN DISTRIBUTION & EXPORTING BAR CHARTS
    for (idx, counter) in ResetCounter::into_iter().enumerate() {
        let cur_counter_data = &dists[idx];
        let counter_mean = dist_mean(cur_counter_data);
        let counter_stdev = dist_stdev(cur_counter_data, counter_mean);

        stats.push(counter_mean);
        stats.push(counter_stdev);

        let offset = Z_SCORE * counter_stdev + counter_mean;
        let bin_vec = construct_bin_vec(0, offset as usize, 100, &dists[idx]);
        export_bar_chart(&graph_out, counter.to_string(), &bin_vec, PLOT_COLORS[idx]);
    }

    // HYPOTHESIS TESTING
}

/*
     // EXECUTION MODE CHECK
    let fname_prefix = release_flag_prefix();

    // RUNNING OUR TESTS
    let (inner_times, outer_times) = run_cycle_tests(TOTAL_OUTER_ITERS, TOTAL_INNER_ITERS, INCREMENT_AMOUNT, RESET_NUMBER);
    times_to_csv(OUTPUT_PATH, (fname_prefix.to_owned() + "inner.csv").as_str(), 
    &inner_times).expect("Couldn't properly send inner data to a csv");
    times_to_csv(OUTPUT_PATH, (fname_prefix.to_owned() + "outer.csv").as_str(), 
    &outer_times).expect("Couldn't properly send outer data to csv");

    // PLOTTING
    plot_reset_counter_graphs(GRAPH_PATH, (fname_prefix.to_owned() + "counter_times").as_str(), &outer_times);

    // CONSTRUCTING BIN DISTRIBUTION AND EXPORTING BAR CHART 
    for (idx, counter) in ResetCounter::into_iter().enumerate() {
        let cur_counter_data = &outer_times[idx];
        let counter_mean = dist_mean(cur_counter_data);
        let counter_stdev = dist_stdev(cur_counter_data, counter_mean);

        //let mut normal = normal_dist(counter_mean, counter_stdev);
        //fill_bin_vec(&mut normal, cur_counter_data.iter().map(|u| *u as usize));
        //println!("{:?}", normal);

        // z = x - mu / stdev -> x = z * stdev + mu
        let offset = Z_SCORE * counter_stdev + counter_mean;
        //let noffset = -1_f32 * Z_SCORE * counter_stdev + counter_mean;
        //println!("z-score = {}, above, below = ({},{})", Z_SCORE, noffset, offset);
        //println!("mean = {}", counter_mean);
        //println!("stdev = {}", counter_stdev);

        // WE WANT BINS WITH A MINUMUM VALUE OF 0 AND MAXIMUM VALUE OF 100~ STDEVS FROM THE MEAN
        let bin_vec = construct_bin_vec(0, offset as usize,  100, &outer_times[idx]);
        export_bar_chart(GRAPH_PATH, counter.to_string(), &bin_vec);
    }
 */
