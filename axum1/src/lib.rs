use once_cell::sync::Lazy;
use regex::Regex;

pub mod cli;
pub mod config;
pub mod email;
pub mod error;
pub mod extractors;
pub mod queue;
pub mod routes;
pub mod search;
pub mod session;
pub mod session_ext;
pub mod sse;
pub mod startup;
pub mod state;
pub mod upload;
pub mod utils;

static RE_USERNAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\.?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ])*$"#).unwrap()
});
static RE_RECIPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[a-zA-Z0-9-íáéúőóüöűÍÁÉÚŐÓÜÖŰ](\-?[a-zA-Z0-9 íáéúőóüöűÍÁÉÚŐÓÜÖŰ - \s])*$"#)
        .unwrap()
});

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{
    sync::{Arc, Mutex},
    task::Waker,
};

#[pin_project::pin_project]
pub struct PausableFuture<F> {
    #[pin]
    future: F,
    state: Arc<Mutex<PausableState>>,
}

impl<F> PausableFuture<F> {
    pub fn new(future: F) -> (Self, Arc<Mutex<PausableState>>) {
        let state = Arc::new(Mutex::new(PausableState::Running));
        let handle = Arc::clone(&state);
        (Self { future, state }, handle)
    }
}

pub struct PausableFutureSupervisor {
    state: Arc<Mutex<PausableState>>,
}

#[derive(Clone)]
pub enum PausableState {
    Running,
    Paused(Waker),
    PausePending,
}

impl PausableFutureSupervisor {
    pub fn new(state: &Arc<Mutex<PausableState>>) -> Self {
        Self {
            state: Arc::clone(state),
        }
    }

    pub fn pause(&mut self) {
        let mut state = self.state.lock().unwrap();

        if let PausableState::Running = state.clone() {
            *state = PausableState::PausePending;
        }
    }

    pub fn resume(&mut self) {
        let mut state = self.state.lock().unwrap();

        if let PausableState::Paused(waker) = state.clone() {
            *state = PausableState::Running;
            waker.wake();
        }
    }
}

impl<F: Future> Future for PausableFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut state = this.state.lock().unwrap();

        match &*state {
            PausableState::PausePending | PausableState::Paused(_) => {
                *state = PausableState::Paused(cx.waker().clone());
                Poll::Pending
            }
            PausableState::Running => {
                drop(state);
                this.future.poll(cx)
            }
        }
    }
}
