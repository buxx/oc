use std::{
    collections::HashMap,
    sync::PoisonError,
    time::{Duration, Instant},
};

use backtrace::Backtrace;
use uuid::Uuid;

pub struct RwLock<T> {
    uuid: Uuid,
    inner: std::sync::RwLock<T>,
    tx: std::sync::mpsc::Sender<(Uuid, Uuid, Backtrace, Instant, RwLockEvent)>,
}

pub struct RwLockReadGuard<'a, T> {
    lock_uuid: Uuid,
    session_uuid: Uuid,
    inner: std::sync::RwLockReadGuard<'a, T>,
    origin: Backtrace,
    tx: std::sync::mpsc::Sender<(Uuid, Uuid, Backtrace, Instant, RwLockEvent)>,
}

pub struct RwLockWriteGuard<'a, T> {
    lock_uuid: Uuid,
    session_uuid: Uuid,
    inner: std::sync::RwLockWriteGuard<'a, T>,
    origin: Backtrace,
    tx: std::sync::mpsc::Sender<(Uuid, Uuid, Backtrace, Instant, RwLockEvent)>,
}

pub enum RwLockEvent {
    AskForRead,
    TakeRead,
    ReleaseRead,
    AskForWrite,
    TakeWrite,
    ReleaseWrite,
}

impl<T> RwLock<T> {
    pub fn new(value: T) -> RwLock<T> {
        let lock = std::sync::RwLock::new(value);

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let prediod = std::env::var("RUST_DEBUG_RW_LOCK_PERIOD").unwrap_or("1".to_string());
            let prediod = prediod.parse::<u64>().unwrap_or(1);
            let prediod = Duration::from_secs(prediod);
            let mut state: HashMap<Uuid, Vec<(Uuid, Backtrace, Instant, RwLockEvent)>> =
                HashMap::new();

            loop {
                match rx.recv_timeout(prediod) {
                    Ok((lock_uuid, session_uuid, backtrace, instant, event)) => match event {
                        RwLockEvent::AskForRead => {
                            state.entry(lock_uuid).or_insert_with(Vec::new).push((
                                session_uuid,
                                backtrace,
                                instant,
                                event,
                            ));
                        }
                        RwLockEvent::TakeRead => {
                            if let Some(events) = state.get_mut(&lock_uuid) {
                                events.retain(|(session_uuid_, _, _, event)| {
                                    session_uuid_ == &session_uuid
                                        && matches!(event, RwLockEvent::AskForRead)
                                })
                            };
                            state.entry(lock_uuid).or_insert_with(Vec::new).push((
                                session_uuid,
                                backtrace,
                                instant,
                                event,
                            ));
                        }
                        RwLockEvent::ReleaseRead => todo!(),
                        RwLockEvent::AskForWrite => todo!(),
                        RwLockEvent::TakeWrite => todo!(),
                        RwLockEvent::ReleaseWrite => todo!(),
                    },
                    Err(_) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        todo!()
                    }
                };
            }
        });

        Self {
            uuid: Uuid::new_v4(),
            inner: lock,
            tx,
        }
    }

    pub fn read(
        &self,
    ) -> Result<RwLockReadGuard<'_, T>, PoisonError<std::sync::RwLockReadGuard<'_, T>>> {
        let backtrace = Backtrace::new();
        let session_uuid = Uuid::new_v4();
        let _ = self.tx.send((
            self.uuid.clone(),
            session_uuid.clone(),
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::AskForRead,
        ));
        let lock = self.inner.read()?;
        let _ = self.tx.send((
            self.uuid.clone(),
            session_uuid.clone(),
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::TakeRead,
        ));
        Ok(RwLockReadGuard {
            lock_uuid: self.uuid.clone(),
            session_uuid,
            inner: lock,
            origin: backtrace,
            tx: self.tx.clone(),
        })
    }

    pub fn write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, T>, PoisonError<std::sync::RwLockWriteGuard<'_, T>>> {
        let backtrace = Backtrace::new();
        let session_uuid = Uuid::new_v4();
        let _ = self.tx.send((
            self.uuid.clone(),
            session_uuid.clone(),
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::AskForWrite,
        ));
        let lock = self.inner.write()?;
        let _ = self.tx.send((
            self.uuid.clone(),
            session_uuid.clone(),
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::TakeWrite,
        ));
        Ok(RwLockWriteGuard {
            lock_uuid: self.uuid.clone(),
            session_uuid,
            inner: lock,
            origin: backtrace,
            tx: self.tx.clone(),
        })
    }
}

impl<'a, T> std::ops::Deref for RwLockReadGuard<'a, T> {
    type Target = std::sync::RwLockReadGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        let _ = self.tx.send((
            self.lock_uuid.clone(),
            self.session_uuid.clone(),
            self.origin.clone(),
            Instant::now(),
            RwLockEvent::ReleaseRead,
        ));
    }
}

impl<'a, T> std::ops::Deref for RwLockWriteGuard<'a, T> {
    type Target = std::sync::RwLockWriteGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        let _ = self.tx.send((
            self.lock_uuid.clone(),
            self.session_uuid.clone(),
            self.origin.clone(),
            Instant::now(),
            RwLockEvent::ReleaseWrite,
        ));
    }
}
