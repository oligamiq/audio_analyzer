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

impl<Input, Output, NOutput>
    MultipleLayers<Input, Output, NOutput>
{
    pub fn add_layer<NewOutput, T: Layer<InputType = Output, OutputType = NewOutput> + 'static>(
        self,
        layer: T,
    ) -> MultipleLayers<Input, Output, NewOutput>
    {
        let tail: MultipleLayersHead<Input, Output, NOutput> = self.head;

        let new_head = MultipleLayersHead {
            tail: tail,
            layer: Some(Box::new(layer)),
            tail_layer: None,
        };

        return MultipleLayers { head: new_head };
    }

    pub fn start(&mut self) {
        let mut head = &mut self.head;
        if let Some(layer) = &head.layer {
            layer.start();
        }



        head.layer = Some(layer);
    }
}

pub struct MultipleLayersHead<Input, Output, NewOutput> {
    tail: Option<Box<MultipleLayersHead<Input, Output, NewOutput>>>,
    layer: Option<Box<dyn Layer<InputType = Output, OutputType = NewOutput>>>,
    tail_layer: Option<Box<dyn Layer<InputType = Input, OutputType = Output>>>,
}
