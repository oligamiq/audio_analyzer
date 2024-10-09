use super::Layer;

pub struct MultipleLayers<Input, Output, NOutput> {
    head: MultipleLayersHead<Input, Output, NOutput>,
}

pub fn layer<Input, Output, T: Layer<InputType = Input, OutputType = Output> + 'static>(
    layer: T,
) -> MultipleLayers<Input, Output, Output> {
    MultipleLayers {
        head: MultipleLayersHead {
            tail: None,
            layer: None,
            tail_layer: Some(Box::new(layer)),
        },
    }
}

impl<Input, Output, NOutput> MultipleLayers<Input, Output, NOutput> {
    pub fn add_layer<NewOutput, T: Layer<InputType = Output, OutputType = NewOutput> + 'static>(
        self,
        layer: T,
    ) -> MultipleLayers<Input, Output, NewOutput> {
        let tail: MultipleLayersHead<Input, Output, NOutput> = self.head;

        let new_head = MultipleLayersHead {
            tail: Some(Box::new(tail)),
            layer: Some(Box::new(layer)),
            tail_layer: None,
        };

        return MultipleLayers { head: new_head };
    }

    pub fn start(&mut self) {
        let mut head = &mut self.head;
        if let Some(layer) = &mut head.layer {
            layer.start();
        } else if let Some(layer) = &mut head.tail_layer {
            layer.start();
        }

        while let Some(tail) = &mut head.tail {
            if let Some(layer) = &mut tail.layer {
                layer.start();
            } else if let Some(layer) = &mut head.tail_layer {
                layer.start();
                break;
            }
            head = tail;
        }
    }
}

pub struct MultipleLayersHead<Input, Output, NewOutput> {
    tail: Option<Box<MultipleLayersHead<Input, Output, NewOutput>>>,
    layer: Option<Box<dyn Layer<InputType = Output, OutputType = NewOutput>>>,
    tail_layer: Option<Box<dyn Layer<InputType = Input, OutputType = Output>>>,
}

impl<Input, Output, NOutput> Layer for MultipleLayers<Input, Output, NOutput> {
    type InputType = Input;

    type OutputType = NOutput;

    fn get_result_stream(&self) -> crossbeam_channel::Receiver<Self::OutputType> {
        // first layer
        let mut head = &self.head;
        if let Some(layer) = &head.layer {
            return layer.get_result_stream();
        } else if let Some(layer) = &head.tail_layer {
            return layer.get_result_stream();
        }

        unreachable!()
    }

    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        let mut handles = Vec::new();
        let mut head = &mut self.head;

        if let Some(layer) = &mut head.layer {
            handles.extend(layer.handle());
        } else if let Some(layer) = &mut head.tail_layer {
            handles.extend(layer.handle());
        }

        while let Some(tail) = &mut head.tail {
            if let Some(layer) = &mut tail.layer {
                handles.extend(layer.handle());
            } else if let Some(layer) = &mut head.tail_layer {
                handles.extend(layer.handle());
                break;
            }
            head = tail;
        }

        handles
    }

    fn set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Self::InputType>) {
        let mut head = &mut self.head;
        if let Some(layer) = &mut head.tail_layer {
            layer.set_input_stream(input_stream);
        }

        while let Some(tail) = &mut head.tail {
            if let Some(layer) = &mut head.tail_layer {
                layer.set_input_stream(input_stream);
                break;
            }
            head = tail;
        }
    }

    fn start(&mut self) {
        self.start();
    }
}
