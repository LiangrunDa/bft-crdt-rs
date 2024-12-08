pub trait BFTCRDT<O> {
    fn interpret_op(&mut self, op: &O);
}