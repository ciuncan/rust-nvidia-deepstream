pub mod observability;

#[derive(Default)]
#[must_use]
pub struct MultiDropGuard(Vec<Box<dyn std::any::Any>>);

impl MultiDropGuard {
    pub fn add(&mut self, droppable: Box<dyn std::any::Any>) {
        self.0.push(droppable)
    }
}

impl Drop for MultiDropGuard {
    fn drop(&mut self) {
        for droppable in self.0.drain(..).rev() {
            drop(droppable)
        }
    }
}
