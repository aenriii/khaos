use std::{fmt, future::Future, sync::Arc};

use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::{di::DI, discord::routers::ResultFuture, Error};

pub trait InteractionFn<T>: 'static {
    fn call(&self, args: Arc<(DI, InteractionCreate)>) -> ResultFuture<T, Error>;
}

pub type InteractionHandlerFn = &'static (dyn InteractionFn<()> + Send + Sync);
pub type InteractionFilterFn = &'static (dyn InteractionFn<bool> + Send + Sync);

impl<F, G, T> InteractionFn<T> for F
where
    F: Fn(Arc<(DI, InteractionCreate)>) -> G + 'static,
    G: Future<Output = Result<T, Error>> + Send + 'static,
{
    fn call(&self, args: Arc<(DI, InteractionCreate)>) -> ResultFuture<T, Error> {
        let fut = (self)(args);
        Box::pin(async move { fut.await })
    }
}

pub struct InteractionHandler {
    filter: InteractionFilterFn,
    handler: InteractionHandlerFn,
}

impl InteractionHandler {
    pub fn new(filter: InteractionFilterFn, handler: InteractionHandlerFn) -> Self {
        Self { filter, handler }
    }
    pub async fn filter(&self, args: Arc<(DI, InteractionCreate)>) -> bool {
        match self.filter.call(args.clone()).await {
            Ok(true) => true,
            _ => false,
        }
    }
    pub async fn handle(&self, args: Arc<(DI, InteractionCreate)>) -> Result<(), Error> {
        let fut = self.handler.call(args.clone());
        let res = fut.await;
        if let Err(_) = res {
            log::error!("Error executing interaction: {}", args.1.id);
        }
        return res;
    }
}
impl fmt::Debug for InteractionHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InteractionHandler")
    }
}
