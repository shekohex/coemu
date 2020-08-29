use dashmap::DashMap;
use network::Actor;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static STATE: OnceCell<State> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct State {
    clients: Arc<DashMap<usize, ClientState>>,
}

impl State {
    /// Init The State.
    /// Should only get called once.
    pub fn init() {
        let state = Self {
            clients: Arc::new(DashMap::new()),
        };
        STATE.set(state).expect("Failed to init the state.");
    }

    /// Get access to the global state.
    pub fn global() -> &'static Self {
        STATE.get().expect(
            "State is uninialized, did you forget to call State::init()!",
        )
    }

    pub fn add_actor(&self, actor: &Actor) {
        self.clients.insert(actor.id(), ClientState::default());
    }

    pub fn remove_actor(&self, actor: &Actor) {
        self.clients.remove(&actor.id());
    }
}

#[derive(Debug, Default)]
struct ClientState {
    pub account_id: u32,
}
