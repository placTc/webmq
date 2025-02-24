pub trait AsyncStart {
    fn start(&mut self) -> impl Future<Output = ()>;
}