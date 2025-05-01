use std::{fmt, pin::Pin, sync::Arc};

use crate::{di::DI, Error};
use futures::future::Future;
use twilight_model::channel::Message;

// magic code adapted from https://github.com/rust-lang/discord-mods-bot/blob/master/src/commands.rs

pub type ResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;
pub type CommandHandler = &'static (dyn CommandFn<()> + Send + Sync);

pub trait CommandFn<T>: 'static {
    fn call(&self, args: Arc<(Message, Vec<String>, DI)>) -> ResultFuture<T, Error>;
}

impl<F, G, T> CommandFn<T> for F
where
    F: Fn(Arc<(Message, Vec<String>, DI)>) -> G + 'static,
    G: Future<Output = Result<T, Error>> + Send + 'static,
{
    fn call(&self, args: Arc<(Message, Vec<String>, DI)>) -> ResultFuture<T, Error> {
        let fut = (self)(args);
        Box::pin(async move { fut.await })
    }
}

pub struct Command {
    pub name: String,
    handler: CommandHandler,
}

impl Command {
    pub fn new(name: String, handler: CommandHandler) -> Self {
        Self { name, handler }
    }
    pub fn execute(&self, msg: Message, args: Vec<String>, di: DI) {
        let fut = self.handler.call(Arc::new((msg, args, di)));
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                log::error!("Error executing command: {}", e);
            }
        });
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command {{ name: {}, handler }}", self.name)
    }
}
