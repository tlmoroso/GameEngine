use anyhow::Result;
use specs::World;
use luminance_glfw::GL33Context;
use crossbeam_epoch::Atomic;
use std::sync::{Arc, Mutex, RwLock};

#[cfg(feature = "trace")]
use tracing::{warn, debug, error, instrument};

pub struct Task<Ret,Args> {
    function: Box<dyn FnOnce(Args) -> Result<Ret>>
}

impl<Ret: 'static, Args: 'static> Task<Ret,Args> {

    #[cfg_attr(feature = "trace", instrument(skip(f)))]
    pub fn new(f: impl FnOnce(Args) -> Result<Ret> + 'static) -> Self {
        Self { function: Box::new(f) }
    }

    pub fn sequence<NewRet: 'static>(self, next: Task<NewRet, Args>) -> Task<NewRet, Args>
        where Args: Clone {
        Task {
            function: Box::new(|args| {
                (self.function)(args.clone())?;
                (next.function)(args)
            })
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, next)))]
    pub fn serialize<OtherRet: 'static>(self, next: Task<OtherRet, (Ret, Args)>) -> Task<OtherRet,Args>
        where Args: Clone {
        Task {
            function: Box::new(|args: Args| {
                let a = (self.function)(args.clone())?;
                let b = (next.function)((a, args))?;
                return Ok(b)
            })
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, other, map)))]
    pub fn join<OtherRet: 'static,NewRet>
    (self, other: Task<OtherRet,Args>, map: impl FnOnce((Ret,OtherRet)) -> NewRet + 'static) -> Task<NewRet,Args>
        where Args: Clone {
        Task {
            function: Box::new(|args: Args| {
                let a = (self.function)(args.clone())?;
                let b = (other.function)(args)?;
                return Ok(map((a,b)))
            })
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, other)))]
    pub fn map<NewRet>(self, other: impl FnOnce(Ret,Args) -> Result<NewRet> + 'static) -> Task<NewRet,Args>
        where Args: Clone {
        Task {
            function: Box::new(|args: Args| {
                let a = (self.function)(args.clone())?;
                return other(a, args)
            })
        }
    }

    #[cfg_attr(feature = "trace", instrument(skip(self, args)))]
    pub fn execute(self, args: Args) -> Result<Ret> {
        (self.function)(args)
    }
}

pub type GenTask<T> = Task<T, Arc<RwLock<World>>>;

pub type DrawTask<T> = Task<T, (Arc<RwLock<World>>, Arc<RwLock<GL33Context>>)>;