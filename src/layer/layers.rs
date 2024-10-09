use super::Layer;

pub struct MultipleLayers<Input, Output, Tail: TailTrait<Input, Output>, NOutput> {
    head: MultipleLayersHead<Input, Output, Tail, NOutput>,
}

pub fn layer<Input, Output, T: Layer<InputType = Input, OutputType = Output> + 'static>(
    layer: T,
) -> MultipleLayers<Input, Output, MultipleLayersTail<Input, Output>, Output> {
    MultipleLayers {
        head: MultipleLayersHead {
            tail: MultipleLayersTail {
                layer: Box::new(layer),
            },
            layer: None,
            __marker: std::marker::PhantomData,
        },
    }
}

impl<Input, Output, Tail: TailTrait<Input, Output>, NOutput>
    MultipleLayers<Input, Output, Tail, NOutput>
{
    pub fn add_layer<NewOutput, T: Layer<InputType = Output, OutputType = NewOutput> + 'static>(
        self,
        layer: T,
    ) -> MultipleLayers<Input, Output, MultipleLayersHead<Input, Output, Tail, NOutput>, NewOutput>
    where
        Tail: TailTrait<Input, Output>,
    {
        let tail: MultipleLayersHead<Input, Output, Tail, NOutput> = self.head;

        let new_head = MultipleLayersHead {
            tail: tail,
            layer: Some(Box::new(layer)),
            __marker: std::marker::PhantomData,
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

pub trait TailTrait<Input, Output> {}

impl<Input, Output, T: TailTrait<Input, Output>, NewOutput> TailTrait<Input, Output>
    for MultipleLayersHead<Input, Output, T, NewOutput>
{
}

pub struct MultipleLayersHead<Input, Output, Tail: TailTrait<Input, Output>, NewOutput> {
    tail: Tail,
    layer: Option<Box<dyn Layer<InputType = Output, OutputType = NewOutput>>>,
    __marker: std::marker::PhantomData<(Input, Output)>,
}

impl<Input, Output, Tail: TailTrait<Input, Output>, NewOutput>
    MultipleLayersHead<Input, Output, Tail, NewOutput>
{
}

pub struct MultipleLayersTail<Input, Output> {
    layer: Box<dyn Layer<InputType = Input, OutputType = Output>>,
}

impl<Input, Output> TailTrait<Input, Output> for MultipleLayersTail<Input, Output> {}
