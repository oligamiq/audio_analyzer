use super::Layer;
use core::panic;
use std::ops::Add;
use std::{any::Any, fmt::Debug};
use typenum::bit::B1;
use typenum::{Integer, PInt, Sum, UInt, UTerm, P1, Z0};

#[derive(Debug)]
pub struct MultipleLayers<Input, OldOutput, Tail: TailTrait<Input, OldOutput>, NOutput, N: Integer>
{
    head: MultipleLayersHead<Input, OldOutput, Tail, NOutput>,
    __marker: std::marker::PhantomData<N>,
}

pub fn layer<
    Input: 'static + Debug,
    Output: 'static + Debug,
    T: Layer<InputType = Input, OutputType = Output> + 'static + Debug,
>(
    layer: T,
) -> MultipleLayers<Input, Output, MultipleLayersTail<Input, Output>, Output, Z0> {
    MultipleLayers {
        head: MultipleLayersHead {
            tail: MultipleLayersTail {
                layer: Box::new(layer),
            },
            layer: None,
            __marker: std::marker::PhantomData,
        },
        __marker: std::marker::PhantomData,
    }
}

impl<
        Input: 'static + Debug,
        OldOutput: 'static + Debug,
        Tail: TailTrait<Input, OldOutput> + 'static + Debug,
        NOutput: 'static + Debug,
        N: Integer + Add<PInt<UInt<UTerm, B1>>> + Debug,
    > MultipleLayers<Input, OldOutput, Tail, NOutput, N>
where
    <N as Add<PInt<UInt<UTerm, B1>>>>::Output: Integer,
{
    pub fn add_layer<
        NewOutput: 'static,
        T: Layer<InputType = NOutput, OutputType = NewOutput> + 'static,
    >(
        self,
        mut layer: T,
    ) -> MultipleLayers<
        Input,
        NOutput,
        MultipleLayersHead<Input, OldOutput, Tail, NOutput>,
        NewOutput,
        Sum<N, P1>,
    >
    where
        Tail: TailTrait<Input, NOutput>,
    {
        layer.set_input_stream(self.get_result_stream());

        let tail: MultipleLayersHead<Input, OldOutput, Tail, NOutput> = self.head;

        let new_head: MultipleLayersHead<
            Input,
            NOutput,
            MultipleLayersHead<Input, OldOutput, Tail, NOutput>,
            NewOutput,
        > = MultipleLayersHead {
            tail: tail,
            layer: Some(Box::new(layer)),
            __marker: std::marker::PhantomData,
        };

        return MultipleLayers {
            head: new_head,
            __marker: std::marker::PhantomData,
        };
    }

    pub fn get_nth<P: 'static>(&self, n: i32) -> Option<&P> {
        self.head.__get_nth(self.get_length() - n - 1)
    }

    pub fn get_length(&self) -> i32 {
        self.head.__get_length()
    }

    pub fn get_0th_layer(&self) -> &dyn Layer<InputType = OldOutput, OutputType = NOutput> {
        let layer = self.head.layer.as_ref().unwrap().as_ref();

        layer
    }
}

pub trait TailTrait<Input, Output>: Debug {
    type LayerInputType: Debug;
    type LayerOutputType: Debug;

    fn as_base(
        &mut self,
    ) -> &mut dyn TailTrait<
        Input,
        Output,
        LayerInputType = Self::LayerInputType,
        LayerOutputType = Self::LayerOutputType,
    >;

    fn __start(&mut self);

    fn __set_input_stream(&mut self, input_stream: crossbeam_channel::Receiver<Input>);

    fn __handle(&mut self) -> Vec<std::thread::JoinHandle<()>>;

    fn __get_nth<P: 'static>(&self, n: i32) -> Option<&P>
    where
        Self: Sized;

    fn __get_length(&self) -> i32;

    fn __get_layer(
        &self,
    ) -> Option<&dyn Layer<InputType = Self::LayerInputType, OutputType = Self::LayerOutputType>>;
}

impl<
        Input: 'static + Debug,
        OldOutput: 'static + Debug,
        T: TailTrait<Input, OldOutput> + Debug,
        NewOutput: 'static + Debug,
    > TailTrait<Input, NewOutput> for MultipleLayersHead<Input, OldOutput, T, NewOutput>
{
    type LayerInputType = OldOutput;
    type LayerOutputType = NewOutput;

    fn as_base(
        &mut self,
    ) -> &mut dyn TailTrait<Input, NewOutput, LayerInputType = OldOutput, LayerOutputType = NewOutput>
    {
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

    fn __get_layer(
        &self,
    ) -> Option<&dyn Layer<InputType = Self::LayerInputType, OutputType = Self::LayerOutputType>>
    {
        if let Some(layer) = &self.layer {
            return Some(layer.as_ref());
        } else {
            return None;
        }
    }
}

#[derive(Debug)]
pub struct MultipleLayersHead<Input, OldOutput, Tail: TailTrait<Input, OldOutput>, NewOutput> {
    tail: Tail,
    layer: Option<Box<dyn Layer<InputType = OldOutput, OutputType = NewOutput> + 'static>>,
    __marker: std::marker::PhantomData<Input>,
}

#[derive(Debug)]
pub struct MultipleLayersTail<Input, Output> {
    layer: Box<dyn Layer<InputType = Input, OutputType = Output>>,
}

impl<Input: 'static + Debug, Output: 'static + Debug> TailTrait<Input, Output>
    for MultipleLayersTail<Input, Output>
{
    type LayerInputType = Input;
    type LayerOutputType = Output;

    fn as_base(
        &mut self,
    ) -> &mut dyn TailTrait<Input, Output, LayerInputType = Input, LayerOutputType = Output> {
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
    fn __get_layer(
        &self,
    ) -> Option<&dyn Layer<InputType = Self::LayerInputType, OutputType = Self::LayerOutputType>>
    {
        return Some(self.layer.as_ref());
    }
}

impl<
        Input: 'static + Debug,
        Output: 'static + Debug,
        Tail: TailTrait<Input, Output> + 'static + Debug,
        NOutput: 'static + Debug,
        N: Integer + Add<PInt<UInt<UTerm, B1>>> + Debug,
    > Layer for MultipleLayers<Input, Output, Tail, NOutput, N>
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
