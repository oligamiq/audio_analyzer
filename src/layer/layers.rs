use std::any::Any;

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
}

pub trait TailTrait<Input, Output> {
    type LayerOutputType;

    fn as_base(
        &mut self,
    ) -> &mut dyn TailTrait<Input, Output, LayerOutputType = Self::LayerOutputType>;

    fn __start(&mut self);

    fn __set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Input>);

    fn __handle(&mut self) -> Vec<std::thread::JoinHandle<()>>;
}

impl<Input, Output, T: TailTrait<Input, Output>, NewOutput> TailTrait<Input, Output>
    for MultipleLayersHead<Input, Output, T, NewOutput>
{
    type LayerOutputType = NewOutput;

    fn as_base(&mut self) -> &mut dyn TailTrait<Input, Output, LayerOutputType = NewOutput> {
        self
    }
    fn __start(&mut self) {
        self.tail.__start();
        self.layer.as_mut().unwrap().start();
    }
    fn __set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Input>) {
        self.tail.__set_input_stream(input_stream);
    }
    fn __handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        let mut handles = self.tail.__handle();
        handles.append(&mut self.layer.as_mut().unwrap().handle());
        handles
    }
}

pub struct MultipleLayersHead<Input, Output, Tail: TailTrait<Input, Output>, NewOutput> {
    tail: Tail,
    layer: Option<Box<dyn Layer<InputType = Output, OutputType = NewOutput>>>,
    __marker: std::marker::PhantomData<Input>,
}

pub struct MultipleLayersTail<Input, Output> {
    layer: Box<dyn Layer<InputType = Input, OutputType = Output>>,
}

impl<Input, Output> TailTrait<Input, Output> for MultipleLayersTail<Input, Output> {
    type LayerOutputType = Output;

    fn as_base(&mut self) -> &mut dyn TailTrait<Input, Output, LayerOutputType = Output> {
        self
    }
    fn __start(&mut self) {
        self.layer.start();
    }
    fn __set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Input>) {
        self.layer.set_input_stream(input_stream);
    }
    fn __handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        self.layer.handle()
    }
}

impl<Input: 'static, Output, Tail: TailTrait<Input, Output> + 'static, NOutput: 'static> Layer
    for MultipleLayers<Input, Output, Tail, NOutput>
{
    type InputType = Input;
    type OutputType = NOutput;

    fn get_result_stream(&self) -> crossbeam_channel::Receiver<Self::OutputType> {
        if let Some(layer) = &self.head.layer {
            layer.get_result_stream()
        } else {
            let any = &self.head.tail as &dyn Any;
            let tail = any
                .downcast_ref::<MultipleLayersTail<Input, NOutput>>()
                .unwrap();

            tail.layer.get_result_stream()
        }
    }

    fn set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Self::InputType>) {
        if let Some(_) = &self.head.layer {
            self.head.tail.__set_input_stream(input_stream);
        } else {
            let any = &mut self.head.tail as &mut dyn Any;
            let tail = any
                .downcast_mut::<MultipleLayersTail<Input, NOutput>>()
                .unwrap();

            tail.layer.set_input_stream(input_stream);
        }
    }

    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        if let Some(_) = &self.head.layer {
            self.head.tail.__handle()
        } else {
            let any = &mut self.head.tail as &mut dyn Any;
            let tail = any
                .downcast_mut::<MultipleLayersTail<Input, NOutput>>()
                .unwrap();

            tail.layer.handle()
        }
    }

    fn start(&mut self) {
        if let Some(_) = &self.head.layer {
            self.head.tail.__start()
        } else {
            let any = &mut self.head.tail as &mut dyn Any;
            let tail = any
                .downcast_mut::<MultipleLayersTail<Input, NOutput>>()
                .unwrap();

            tail.layer.start()
        }
    }
}
