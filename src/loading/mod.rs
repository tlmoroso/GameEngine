use anyhow::Result;
use specs::World;

pub struct Task<T> {
    function: Box<dyn FnOnce(&mut World) -> Result<T>>
}

impl<T: 'static> Task<T> {

    pub fn new(f: impl FnOnce(&mut World) -> Result<T> + 'static) -> Self {
        Self { function: Box::new(f) }
    }

    pub fn join<V: 'static,Y>(self, other: Task<V>, map: impl FnOnce((T,V)) -> Y + 'static) -> Task<Y> {
        Task {
            function: Box::new(|w: &mut World| {
                let a = (self.function)(w)?;
                let b = (other.function)(w)?;
                return Ok(map((a,b)))
            })
        }
    }

    pub fn map<V>(self, other: impl FnOnce(T, &mut World) -> Result<V> + 'static) -> Task<V> {
        Task {
            function: Box::new(|w: &mut World| {
                let a = (self.function)(w)?;
                return other(a, w)
            })
        }
    }
}