pub trait CRDT<O> {
    fn interpret_op(&mut self, op: &O);
}