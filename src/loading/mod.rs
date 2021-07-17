use anyhow::Result;
use specs::World;
use luminance_glfw::GL33Context;
use crossbeam_epoch::Atomic;
use std::sync::{Arc, Mutex, RwLock};

pub struct Task<Ret,Args> {
    function: Box<dyn FnOnce(Args) -> Result<Ret>>
}

impl<Ret: 'static, Args: 'static + Clone> Task<Ret,Args> {

    pub fn new(f: impl FnOnce(Args) -> Result<Ret> + 'static) -> Self {
        Self { function: Box::new(f) }
    }

    pub fn sequence<OtherRet: 'static,NewRet>(self, next: Task<OtherRet, Args>,
      map: impl FnOnce((Ret, OtherRet)) -> NewRet + 'static) -> Task<NewRet,Args> {
        Task {
            function: Box::new(|args: Args| {
                let a = (self.function)(args.clone())?;
                let b = (next.function)(args)?;
                return Ok(map((a,b)))
            })
        }
    }

    pub fn join<OtherRet: 'static,NewRet>
    (self, other: Task<OtherRet,Args>, map: impl FnOnce((Ret,OtherRet)) -> NewRet + 'static) -> Task<NewRet,Args> {
        Task {
            function: Box::new(|args: Args| {
                let a = (self.function)(args.clone())?;
                let b = (other.function)(args)?;
                return Ok(map((a,b)))
            })
        }
    }

    pub fn map<NewRet>(self, other: impl FnOnce(Ret,Args) -> Result<NewRet> + 'static) -> Task<NewRet,Args> {
        Task {
            function: Box::new(|args: Args| {
                let a = (self.function)(args.clone())?;
                return other(a, args)
            })
        }
    }

    pub fn execute(self, args: Args) -> Result<Ret> {
        (self.function)(args)
    }
}

pub type GenTask<T> = Task<T, Arc<RwLock<World>>>;

pub type DrawTask<T> = Task<T, (Arc<RwLock<World>>, Arc<RwLock<GL33Context>>)>;