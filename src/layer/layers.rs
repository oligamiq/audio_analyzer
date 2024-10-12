use clap::builder::Str;
use color_eyre::eyre::{eyre, ContextCompat};

use super::Layer;
use crate::{utils::debug, Result};
use std::{any::Any, fmt::Debug};

#[derive(Debug)]
pub struct MultipleLayers {
    layers: Vec<Box<dyn Layer>>,
}

impl Default for MultipleLayers {
    fn default() -> Self {
        Self { layers: Vec::new() }
    }
}

impl MultipleLayers {
    pub fn new(layers: Vec<Box<dyn Layer>>) -> Self {
        Self { layers }
    }

    pub fn get_length(&self) -> usize {
        self.layers.len()
    }

    pub fn get_layer(&self, index: usize) -> Option<&Box<dyn Layer>> {
        self.layers.get(index)
    }

    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Box<dyn Layer>> {
        self.layers.get_mut(index)
    }

    pub fn push_boxed_layer(&mut self, layer: Box<dyn Layer>) {
        self.layers.push(layer);
    }

    pub fn push_layer(&mut self, layer: impl Layer + 'static) {
        self.layers.push(Box::new(layer));
    }

    pub fn pop_layer(&mut self) -> Option<Box<dyn Layer>> {
        self.layers.pop()
    }

    pub fn insert_layer(&mut self, index: usize, layer: Box<dyn Layer>) {
        self.layers.insert(index, layer);
    }

    pub fn remove_layer(&mut self, index: usize) -> Option<Box<dyn Layer>> {
        if self.layers.len() <= index {
            return None;
        }
        Some(self.layers.remove(index))
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<Box<dyn Layer>> {
        self.layers.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Layer>> {
        self.layers.iter_mut()
    }

    pub fn check_types(&self) -> Result<()> {
        let mut input_type = self
            .layers
            .first()
            .with_context(|| "MultipleLayers must have at least one layer to check types")?
            .input_type();
        for (i, layer) in self.layers.iter().enumerate() {
            if input_type != layer.input_type() {
                return Err(eyre!(
                    "Layer {i}'s input type is not matched with previous layer's output type\n\
                    Expected: {input_type}\n\
                    Actual: {}",
                    layer.input_type()
                ));
            }
            input_type = layer.output_type();
        }
        Ok(())
    }
}

impl Layer for MultipleLayers {
    fn through(&mut self, input: &dyn Any) -> Result<Vec<Box<(dyn Any + 'static)>>> {
        let ret = Ok(self
            .layers
            .iter_mut()
            .try_fold(None::<Vec<Box<dyn Any>>>, move |inputs, layer| {
                if let Some(inputs) = inputs {
                    let mut outputs = Vec::new();
                    for input in inputs {
                        outputs.extend(layer.through(input.as_ref())?);
                    }

                    // tracing::debug!("layer: {:?}, outputs: {:?}", layer, outputs);

                    return Ok::<Option<Vec<Box<dyn Any>>>, color_eyre::Report>(Some(outputs));
                } else {
                    Ok(Some(layer.through(input)?))
                }
            })?
            .unwrap());

        // tracing::debug!("ret: {:?}", ret);

        ret
    }

    fn input_type(&self) -> &'static str {
        self.layers.first().map_or("", |layer| layer.input_type())
    }

    fn output_type(&self) -> &'static str {
        self.layers.last().map_or("", |layer| layer.output_type())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
