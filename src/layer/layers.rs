use core::panic;
use std::any::Any;

use super::Layer;

pub struct MultipleLayers<Input, Output, Tail: TailTrait<Input, Output>, NOutput> {
    head: MultipleLayersHead<Input, Output, Tail, NOutput>,
}

pub fn layer<
    Input: 'static,
    Output: 'static,
    T: Layer<InputType = Input, OutputType = Output> + 'static,
>(
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

impl<
        Input: 'static,
        Output: 'static,
        Tail: TailTrait<Input, Output> + 'static,
        NOutput: 'static,
    > MultipleLayers<Input, Output, Tail, NOutput>
{
    pub fn add_layer<
        NewOutput: 'static,
        T: Layer<InputType = NOutput, OutputType = NewOutput> + 'static,
    >(
        self,
        layer: T,
    ) -> MultipleLayers<Input, NOutput, MultipleLayersHead<Input, Output, Tail, NOutput>, NewOutput>
    where
        Tail: TailTrait<Input, NOutput>,
    {
        let tail: MultipleLayersHead<Input, Output, Tail, NOutput> = self.head;
        // let tail = &tail as &dyn TailTrait<Input, NOutput, LayerOutputType = NOutput>;

        let new_head: MultipleLayersHead<
            Input,
            NOutput,
            MultipleLayersHead<Input, Output, Tail, NOutput>,
            NewOutput,
        > = MultipleLayersHead {
            tail: tail,
            layer: Some(Box::new(layer)),
            __marker: std::marker::PhantomData,
        };

        return MultipleLayers { head: new_head };
    }

    pub fn get_nth<P: 'static>(&self, n: i32) -> Option<&P> {
        self.head.__get_nth(self.get_length() - n - 1)
    }

    pub fn get_length(&self) -> i32 {
        self.head.__get_length()
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

    fn __get_nth<P: 'static>(&self, n: i32) -> Option<&P>
    where
        Self: Sized;

    fn __get_length(&self) -> i32;
}

impl<Input: 'static, Output: 'static, T: TailTrait<Input, Output>, NewOutput: 'static>
    TailTrait<Input, NewOutput> for MultipleLayersHead<Input, Output, T, NewOutput>
{
    type LayerOutputType = NewOutput;

    fn as_base(&mut self) -> &mut dyn TailTrait<Input, NewOutput, LayerOutputType = NewOutput> {
        self
    }
    fn __start(&mut self) {
        self.tail.__start();
        if let Some(layer) = &mut self.layer {
            layer.start();
        }
    }
    fn __set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Input>) {
        self.tail.__set_input_stream(input_stream);
    }
    fn __handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        let mut handles = self.tail.__handle();
        if let Some(layer) = &mut self.layer {
            handles.append(&mut layer.handle());
        }
        handles
    }

    fn __get_nth<P: 'static>(&self, n: i32) -> Option<&P>
    where
        Self: Sized,
    {
        if n == 0 {
            if let Some(layer) = &self.layer {
                let any_layer = layer.as_any();
                let layer = any_layer.downcast_ref::<P>();
                if let Some(layer) = layer {
                    return Some(layer);
                } else {
                    panic!(
                        "Layer type mismatch: expected {:?}",
                        std::any::type_name::<P>(),
                    );
                }
            } else {
                return self.tail.__get_nth(n);
            }
        } else {
            return self.tail.__get_nth(n - 1);
        }
    }

    fn __get_length(&self) -> i32 {
        if let Some(_) = &self.layer {
            return 1 + self.tail.__get_length();
        } else {
            return self.tail.__get_length();
        }
    }
}

pub struct MultipleLayersHead<Input, Output, Tail: TailTrait<Input, Output>, NewOutput> {
    tail: Tail,
    layer: Option<Box<dyn Layer<InputType = Output, OutputType = NewOutput> + 'static>>,
    __marker: std::marker::PhantomData<Input>,
}

pub struct MultipleLayersTail<Input, Output> {
    layer: Box<dyn Layer<InputType = Input, OutputType = Output>>,
}

impl<Input: 'static, Output: 'static> TailTrait<Input, Output>
    for MultipleLayersTail<Input, Output>
{
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
    fn __get_nth<P: 'static>(&self, n: i32) -> Option<&P>
    where
        Self: Sized,
    {
        if n == 0 {
            let any_layer = self.layer.as_any();
            let layer = any_layer.downcast_ref::<P>();
            if let Some(layer) = layer {
                return Some(layer);
            } else {
                panic!(
                    "Layer type mismatch: expected {:?}",
                    std::any::type_name::<P>(),
                );
            }
        } else {
            return None;
        }
    }
    fn __get_length(&self) -> i32 {
        return 1;
    }
}

impl<
        Input: 'static,
        Output: 'static,
        Tail: TailTrait<Input, Output> + 'static,
        NOutput: 'static,
    > Layer for MultipleLayers<Input, Output, Tail, NOutput>
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}
