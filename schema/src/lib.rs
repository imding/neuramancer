mod note;

pub use note::*;

pub trait SurrealRecord {
    fn put_id(&mut self);
}
