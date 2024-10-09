pub mod layers;

use crossbeam_channel::Receiver;

pub trait Layer {
    type InputType;
    type OutputType;

    fn get_result_stream(&self) -> Receiver<Self::OutputType>;
    fn set_input_stream(&mut self, input_stream: Receiver<Self::InputType>);
    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>>;
    fn start(&mut self);
}
