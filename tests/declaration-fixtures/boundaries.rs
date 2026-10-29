pub mod boundary {
    pub struct Worker {
        value: i32,
    }
    pub enum State {
        Ready,
    }
    pub trait Runner {
        fn step(&self);
    }
    pub type Count = usize;
    pub fn execute() {
    }
}
