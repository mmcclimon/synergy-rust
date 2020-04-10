pub mod slack;

pub trait Channel {
    fn start(&self);
}
