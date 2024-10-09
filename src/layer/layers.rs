use std::any::{Any, TypeId};

use crossbeam_channel::Receiver;

use super::Layer;

pub struct MultipleLayers<Input, Output> {
    layers: Vec<Box<dyn Layer<InputType = dyn Any, OutputType = dyn Any>>>,
    __before_out_type_id: Option<TypeId>,
    __before_out_type_name: Option<String>,
    __phantom: std::marker::PhantomData<(Input, Output)>,
}

impl<Input, Output> MultipleLayers<Input, Output> {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            __before_out_type_id: None,
            __before_out_type_name: None,
            __phantom: std::marker::PhantomData,
        }
    }

    pub fn add_layer<U, V, T: Layer<InputType = U, OutputType = V> + 'static>(&mut self, layer: T) {
        if let Some(__before_out_type_id) = self.__before_out_type_id {
            let input_type_id = TypeId::of::<U>();
            if __before_out_type_id != input_type_id {
                panic!(
                    "Layer type mismatch, expected: {:?}, got: {:?}",
                    self.__before_out_type_name.as_ref().unwrap(),
                    std::any::type_name::<U>(),
                );
            }

            // layer.set_input_stream(self.layers.last().unwrap().get_result_stream() as Receiver<U>);
            let result_stream = self.layers.last().unwrap().get_result_stream();
            let input_stream = result_stream.
            layer.set_input_stream(
        } else {
            if TypeId::of::<Input>() != TypeId::of::<U>() {
                panic!(
                    "Layer type mismatch, expected: {:?}, got: {:?}",
                    std::any::type_name::<Input>(),
                    std::any::type_name::<U>(),
                );
            }
        }
        self.__before_out_type_id = Some(TypeId::of::<V>());
        self.__before_out_type_name = Some(std::any::type_name::<V>().to_string());

        self.layers.push(Box::new(layer));
    }

    pub fn get_last_result_stream(&self) -> Receiver<Output> {
        let __before_out_type_id = self.__before_out_type_id.expect("No layer added yet");

        if TypeId::of::<Output>() != __before_out_type_id {
            panic!(
                "Layer type mismatch, expected: {:?}, last layer output is: {:?}",
                std::any::type_name::<Output>(),
                self.__before_out_type_name.as_ref().unwrap(),
            );
        }

        self.layers.last().unwrap().get_result_stream()
    }
}

impl<Input, Output> Layer for MultipleLayers<Input, Output> {
    type InputType = Input;

    type OutputType = Output;

    fn get_result_stream(&self) -> Receiver<Self::OutputType> {
        self.get_last_result_stream()
    }

    fn set_input_stream(&mut self, input_stream: Receiver<Self::InputType>) {
        if let Some(layer) = self.layers.first_mut() {
            layer.set_input_stream(input_stream);
        } else {
            panic!("No layer added yet");
        }
    }

    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        let mut handles = Vec::new();

        for layer in self.layers.iter_mut() {
            handles.extend(layer.handle());
        }

        handles
    }

    fn start(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.start();
        }
    }
}
