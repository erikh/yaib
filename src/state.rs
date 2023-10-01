#![allow(dead_code)]
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub opened: Vec<String>,
}

pub type ProtectedState = Arc<Mutex<State>>;
