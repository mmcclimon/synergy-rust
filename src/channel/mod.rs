pub mod slack;

pub trait Channel {
    fn start(&mut self);
}
