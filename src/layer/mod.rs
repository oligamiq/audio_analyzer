pub mod layers;

use crate::Result;
use std::{any::Any, fmt::Debug};

pub trait Layer: Debug {
    fn through(&mut self, input: &dyn Any) -> Result<Vec<Box<dyn Any>>>;
    fn input_type(&self) -> &'static str;
    fn output_type(&self) -> &'static str;
    fn as_any(&self) -> &dyn std::any::Any;
}
