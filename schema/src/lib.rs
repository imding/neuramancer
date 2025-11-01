mod knot;
mod note;

pub use {knot::*, note::*};

pub trait SurrealRecord {
    fn put_id(&mut self);
}
