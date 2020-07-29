use actix::prelude::*;
use actix_cache::dev::{Backend, BackendError, Delete, DeleteStatus, Get, Lock, LockStatus, Set};
use actix_cache::{Cache, CacheError, Cacheable};
use serde::{Deserialize, Serialize};

struct UpstreamActor;

impl Actor for UpstreamActor {
    type Context = Context<Self>;
}

#[derive(MessageResponse, Deserialize, Serialize, Debug)]
struct Pong(i32);

impl Cacheable for Ping {
    fn cache_key(&self) -> Result<String, CacheError> {
        Ok(format!("Ping::{}", self.id))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Pong, ()>")]
struct Ping {
    pub id: i32,
}

impl Handler<Ping> for UpstreamActor {
    type Result = ResponseFuture<<Ping as Message>::Result>;

    fn handle(&mut self, msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(async move {
            actix_rt::time::delay_for(core::time::Duration::from_secs(3)).await;
            Ok(Pong(msg.id))
        })
    }
}

struct DummySyncBackend;

impl Actor for DummySyncBackend {
    type Context = SyncContext<Self>;
}

impl Backend for DummySyncBackend {
    type Actor = Self;
    type Context = SyncContext<Self>;
}

impl Handler<Get> for DummySyncBackend {
    type Result = Result<Option<String>, BackendError>;

    fn handle(&mut self, _msg: Get, _: &mut Self::Context) -> Self::Result {
        log::warn!("Dummy sync backend GET");
        Ok(None)
    }
}

impl Handler<Set> for DummySyncBackend {
    type Result = Result<String, BackendError>;

    fn handle(&mut self, _msg: Set, _: &mut Self::Context) -> Self::Result {
        log::warn!("Dummy sync backend SET");
        Ok("42".to_owned())
    }
}

impl Handler<Delete> for DummySyncBackend {
    type Result = Result<DeleteStatus, BackendError>;

    fn handle(&mut self, _msg: Delete, _: &mut Self::Context) -> Self::Result {
        log::warn!("Dummy sync backend Delete");
        Ok(DeleteStatus::Missing)
    }
}

impl Handler<Lock> for DummySyncBackend {
    type Result = Result<LockStatus, BackendError>;

    fn handle(&mut self, _msg: Lock, _: &mut Self::Context) -> Self::Result {
        log::warn!("Dummy sync backend Lock");
        Ok(LockStatus::Acquired)
    }
}

#[actix_rt::main]
async fn main() -> Result<(), CacheError> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let dummy_sync_backend = { SyncArbiter::start(3, move || DummySyncBackend) };

    let cache = Cache::builder().build(dummy_sync_backend).start();
    let upstream = UpstreamActor.start();

    let msg = Ping { id: 42 };
    let res = cache.send(msg.into_cache(upstream)).await??;
    dbg!(res.unwrap());

    Ok(())
}