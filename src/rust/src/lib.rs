//use extendr_api::prelude::*;
use extendr_api::{extendr, extendr_module};
use ndarray::{ArrayView1, ArrayView2, indices_of, Array2, s, par_azip, Axis};
use ndarray::parallel::prelude::*;
use ndarray_stats::{DeviationExt};

// Calculate Euclidean distance matrix
#[extendr]
fn d_bar(a: ArrayView2<f64>) -> f64 {
    let dists = distmat(&a, &a);
    // mean of the distance matrix after dropping 0s
    (dists.sum() as f64) / ((dists.nrows() * dists.nrows() - dists.nrows()) as f64)
}

fn distmat(
    data: &ArrayView2<f64>,
    query: &ArrayView2<f64>,
) -> Array2<f64> {
    let mut distances = Array2::zeros((query.nrows(), data.nrows()));
    let idx = indices_of(&distances);
    let func = |i: usize, j: usize| -> f64 {
        query.slice(s![i, ..]).l2_dist(&data.slice(s![j, ..])).unwrap()
    };
    par_azip!((c in &mut distances, (i,j) in idx){*c = func(i,j)});
    return distances
}

// Find minimum distance between two data sets
#[extendr]
fn min_dists(
    data: ArrayView2<f64>,
    query: ArrayView2<f64>,
    distinct: bool,
) -> Vec<f64> {
  let mut dists = distmat(&data, &query);
  if !distinct {
    let mut diag = dists.diag_mut();
    diag.fill(f64::INFINITY);
  }
  dists.map_axis(Axis(1), |view| view.into_par_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().to_owned()).to_vec()
}

// Calculate geometric mean functional relationship parameters
#[extendr]
fn gmfr_rust(truth: ArrayView1<f64>, estimate: ArrayView1<f64>, corsign: i8) -> [f64; 2] {
    let mean_truth = truth.mean().unwrap();
    let mean_estimate = estimate.mean().unwrap();

    let b = ((truth.into_owned() - mean_truth).mapv(|a| a.powi(2)).sum() /
        (estimate.into_owned() - mean_estimate).mapv(|a| a.powi(2)).sum()).sqrt().abs() * corsign as f64;

    let a = mean_truth - (b * mean_estimate);

    [a, b]
}

// Calculate ssd
#[extendr]
fn ssd_rust(truth: ArrayView1<f64>, estimate: ArrayView1<f64>) -> [f64; 1] {
    [(truth.into_owned() - estimate.into_owned()).mapv(|a| a.powi(2)).sum()]
}

// Sum of Potential Difference from Ji and Gallo (2006)
#[extendr]
fn spod_rust(truth: ArrayView1<f64>, estimate: ArrayView1<f64>) -> [f64; 1] {
    let mean_truth = truth.mean().unwrap();
    let mean_estimate = estimate.mean().unwrap();
    let term_1 = (mean_truth - mean_estimate).abs();

    [((term_1 + (truth.into_owned() - mean_truth).mapv(|a| a.abs())) *
    (term_1 + (estimate.into_owned() - mean_estimate).mapv(|a| a.abs()))).sum()]
}

// Return the unsystematic sum product-difference from Ji and Gallo (2006)
#[extendr]
fn spdu_rust(truth: ArrayView1<f64>, estimate: ArrayView1<f64>, corsign: i8) -> [f64; 1] {
    let gmfr_predict_truth = gmfr_rust(truth, estimate, corsign);
    let gmfr_predict_estimate = gmfr_rust(estimate, truth, corsign);

    let owned_estimate = estimate.into_owned();
    let owned_truth = truth.into_owned();

    let predicted_truth = gmfr_predict_truth[0] + (gmfr_predict_truth[1] * &owned_estimate);
    let predicted_estimate = gmfr_predict_estimate[0] + (gmfr_predict_estimate[1] * &owned_truth);

    [((owned_estimate - predicted_estimate).mapv(|a| a.abs()) *
    (owned_truth - predicted_truth).mapv(|a| a.abs())).sum()]
}

// Return the systematic sum product-difference from Ji and Gallo (2006)
#[extendr]
fn spds_rust(truth: ArrayView1<f64>, estimate: ArrayView1<f64>, corsign: i8) -> [f64; 1] {
  [ssd_rust(truth, estimate)[0] - spdu_rust(truth, estimate, corsign)[0]]
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod waywiserrr;
    fn d_bar;
    fn gmfr_rust;
    fn ssd_rust;
    fn spod_rust;
    fn spdu_rust;
    fn spds_rust;
    fn min_dists;
}
