use ndarray::{Array1, Array2};
use num_complex::Complex;

pub enum DeferredReturn<'a> {
    VecF32(&'a Vec<f32>),
    Array1ComplexF64(&'a Array1<Complex<f64>>),
    Array2F64(&'a Array2<f64>),
    Array1F64F64(&'a Array1<(f64, f64)>),
}

impl<'a> DeferredReturn<'a> {
    pub fn from_boxed_any(input: &'a Box<dyn std::any::Any>) -> Self {
        if let Some(data) = input.downcast_ref::<Vec<f32>>() {
            Self::VecF32(data)
        } else if let Some(data) = input.downcast_ref::<Array1<Complex<f64>>>() {
            Self::Array1ComplexF64(data)
        } else if let Some(data) = input.downcast_ref::<Array1<(f64, f64)>>() {
            Self::Array1F64F64(data)
        } else if let Some(data) = input.downcast_ref::<Array2<f64>>() {
            Self::Array2F64(data)
        } else {
            panic!("Invalid type");
        }
    }
}
