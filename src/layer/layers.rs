use super::Layer;
use core::panic;
use std::{any::Any, fmt::Debug};

#[derive(Debug)]
pub struct MultipleLayers<Input, OldOutput, Tail: TailTrait<Input, OldOutput>, NOutput> {
    head: MultipleLayersHead<Input, OldOutput, Tail, NOutput>,
}

pub fn layer<
    Input: 'static + Debug,
    Output: 'static + Debug,
    T: Layer<InputType = Input, OutputType = Output> + 'static + Debug,
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
        Input: 'static + Debug,
        OldOutput: 'static + Debug,
        Tail: TailTrait<Input, OldOutput> + 'static + Debug + AsAny,
        NOutput: 'static + Debug,
    > MultipleLayers<Input, OldOutput, Tail, NOutput>
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

        return MultipleLayers { head: new_head };
    }

    pub fn get_nth<P: 'static>(&self, n: i32) -> Option<&P> {
        self.head.__get_nth(self.get_length() - n - 1)
    }

    pub fn get_length(&self) -> i32 {
        self.head.__get_length()
    }

    pub fn get_0th_layer(&self) -> Option<&dyn Layer<InputType = OldOutput, OutputType = NOutput>> {
        self.head.__get_layer()
    }

    pub fn get_tail(&self) -> &Tail {
        &self.head.tail
    }
}

pub trait TailTrait<Input, Output>: Debug + Sized {
    type LayerInputType: Debug;
    type LayerOutputType: Debug;
    type Next: Debug + AsAny + TailTrait<Input, Self::LayerInputType>;
    type NextLayerInputType: Debug;

    // fn as_base(
    //     &mut self,
    // ) -> &mut dyn TailTrait<
    //     Input,
    //     Output,
    //     LayerInputType = Self::LayerInputType,
    //     LayerOutputType = Self::LayerOutputType,
    //     Next = Self::Next,
    //     NextLayerInputType = Self::NextLayerInputType,
    // >;

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

    fn __get_tail(&self) -> Option<&Self::Next>;
}

impl<
        Input: 'static + Debug,
        OldOutput: 'static + Debug,
        T: TailTrait<Input, OldOutput> + Debug + AsAny,
        NewOutput: 'static + Debug,
    > TailTrait<Input, NewOutput> for MultipleLayersHead<Input, OldOutput, T, NewOutput>
{
    type LayerInputType = OldOutput;
    type LayerOutputType = NewOutput;
    type Next = T;
    type NextLayerInputType = T::LayerInputType;

    // fn as_base(
    //     &mut self,
    // ) -> &mut dyn TailTrait<
    //     Input,
    //     NewOutput,
    //     LayerInputType = OldOutput,
    //     LayerOutputType = NewOutput,
    //     Next = T,
    //     NextLayerInputType = T::LayerInputType,
    // > {
    //     self
    // }
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

    fn __get_tail(&self) -> Option<&Self::Next> {
        return Some(&self.tail);
    }
}

#[derive(Debug)]
pub struct MultipleLayersHead<Input, OldOutput, Tail: TailTrait<Input, OldOutput>, NewOutput> {
    tail: Tail,
    layer: Option<Box<dyn Layer<InputType = OldOutput, OutputType = NewOutput> + 'static>>,
    __marker: std::marker::PhantomData<Input>,
}

impl<
        Input: 'static + Debug,
        OldOutput: 'static + Debug,
        T: TailTrait<Input, OldOutput> + Debug + AsAny + 'static,
        NewOutput: 'static + Debug,
    > AsAny for MultipleLayersHead<Input, OldOutput, T, NewOutput>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct MultipleLayersTail<Input, Output> {
    layer: Box<dyn Layer<InputType = Input, OutputType = Output>>,
}

impl<Input: 'static + Debug, Output: 'static + Debug> AsAny for MultipleLayersTail<Input, Output> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Input: 'static + Debug, Output: 'static + Debug> TailTrait<Input, Output>
    for MultipleLayersTail<Input, Output>
{
    type LayerInputType = Input;
    type LayerOutputType = Output;
    type Next = ();
    type NextLayerInputType = ();

    // fn as_base(
    //     &mut self,
    // ) -> &mut dyn TailTrait<
    //     Input,
    //     Output,
    //     LayerInputType = Input,
    //     LayerOutputType = Output,
    //     Next = (),
    //     NextLayerInputType = (),
    // > {
    //     self
    // }
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
    fn __get_tail(&self) -> Option<&Self::Next> {
        return None;
    }
}

impl<
        Input: 'static + Debug,
        Output: 'static + Debug,
        Tail: TailTrait<Input, Output> + 'static + Debug,
        NOutput: 'static + Debug,
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
        if let Some(layer) = &mut self.head.layer {
            trace!("Starting layer");

            layer.start();
            self.head.tail.__start();
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

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl AsAny for () {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Input, Output> TailTrait<Input, Output> for () {
    type LayerInputType = ();
    type LayerOutputType = ();
    type Next = ();
    type NextLayerInputType = ();

    // fn as_base(
    //     &mut self,
    // ) -> &mut dyn TailTrait<(), (), LayerInputType = (), LayerOutputType = (), Next = (), NextLayerInputType = ()> {
    //     self
    // }
    fn __start(&mut self) {}
    fn __set_input_stream(&mut self, _input_stream: crossbeam_channel::Receiver<Input>) {}
    fn __handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        Vec::new()
    }
    fn __get_nth<P: 'static>(&self, _n: i32) -> Option<&P>
    where
        Self: Sized,
    {
        None
    }
    fn __get_length(&self) -> i32 {
        0
    }
    fn __get_layer(
        &self,
    ) -> Option<&dyn Layer<InputType = Self::LayerInputType, OutputType = Self::LayerOutputType>>
    {
        None
    }
    fn __get_tail(&self) -> Option<&Self::Next> {
        None
    }
}

macro_rules! LayerCallFunc {
    (
        @loop_inner,
        $multi_layers:expr,
        $func:ident,
        $tail:ident
    ) => {
        let tail = $tail.__get_tail().unwrap();
        let layer = tail.__get_layer();
        if layer.is_none() {
            let layer = tail.__get_tail().unwrap().__get_layer().unwrap();
            $func(layer);

            return;
        }
        let layer = layer.unwrap();

        $func(layer);
    };
    (
        $multi_layers:expr,
        $func:ident
    ) => {
        {
            LayerCallFunc!($multi_layers, $func, 10)
        }
    };
    (
        $multi_layers:expr,
        $func:ident,
        10
    ) => {
        {
            let _____func = || {
                let layer = $multi_layers.get_0th_layer();
                if layer.is_none() {
                    let layer = $multi_layers.get_tail().__get_layer().unwrap();

                    $func(layer);

                    return;
                }
                let layer = layer.unwrap();

                $func(layer);

                let tail = $multi_layers.get_tail();
                let layer = tail.__get_layer();
                if layer.is_none() {
                    let layer = tail.__get_tail().unwrap().__get_layer().unwrap();
                    $func(layer);

                    return;
                }
                let layer = layer.unwrap();

                $func(layer);

                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
                LayerCallFunc!(@loop_inner, $multi_layers, $func, tail);
            };
            _____func();
        }
    };
}

use tracing::trace;
pub(crate) use LayerCallFunc;
