use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum RequestPayload {
    GenerateLoginToken {
        account_id: u32,
        realm_id: u32,
    },
    RemoveLoginToken {
        token: u64,
    },
    StoreCreationToken {
        token: u32,
        account_id: u32,
        realm_id: u32,
    },
    RemoveCreationToken {
        token: u32,
    },
}

#[derive(Debug)]
struct Request {
    pub payload: RequestPayload,
    pub response: oneshot::Sender<Response>,
}

#[derive(Debug, Clone)]
enum Response {
    Ok,
    LoginToken { token: u64 },
    LoginTokenRemoved { value: Option<LoginToken> },
    CreationTokenRemoved { value: Option<CreationToken> },
}

#[derive(Clone, Debug)]
pub struct TokenStore {
    to_worker: mpsc::Sender<Request>,
    worker_handle: Arc<tokio::task::JoinHandle<()>>,
}

struct TokenStoreWorker {
    login_tokens: HashMap<u64, LoginToken>,
    creation_tokens: HashMap<u32, CreationToken>,
    from_controller: mpsc::Receiver<Request>,
}

#[derive(Clone, Debug)]
pub struct LoginToken {
    pub account_id: u32,
    pub realm_id: u32,
}

#[derive(Clone, Debug)]
pub struct CreationToken {
    pub account_id: u32,
    pub realm_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeneratedLoginToken {
    pub token: u64,
}

impl TokenStore {
    /// Construct new TokenStore.
    pub fn new() -> Self {
        let (to_worker, from_controller) = mpsc::channel(32);
        let worker = TokenStoreWorker {
            login_tokens: Default::default(),
            creation_tokens: Default::default(),
            from_controller,
        };
        let worker_handle = tokio::spawn(worker.run());
        Self {
            to_worker,
            worker_handle: Arc::new(worker_handle),
        }
    }

    /// Generate a new Login Token.
    ///
    /// The token will be stored internally, and can be later removed by calling
    /// [`TokenStore::remove_login_token`].
    pub async fn generate_login_token(
        &self,
        account_id: u32,
        realm_id: u32,
    ) -> Result<GeneratedLoginToken, crate::Error> {
        let (tx, rx) = oneshot::channel();
        let request = Request {
            payload: RequestPayload::GenerateLoginToken {
                account_id,
                realm_id,
            },
            response: tx,
        };
        self.to_worker.send(request).await?;
        let response = rx.await?;
        match response {
            Response::LoginToken { token } => Ok(GeneratedLoginToken { token }),
            _ => unreachable!("invalid response"),
        }
    }

    /// Remove a Login Token.
    pub async fn remove_login_token(
        &self,
        token: u64,
    ) -> Result<Option<LoginToken>, crate::Error> {
        let (tx, rx) = oneshot::channel();
        let request = Request {
            payload: RequestPayload::RemoveLoginToken { token },
            response: tx,
        };
        self.to_worker.send(request).await?;
        let response = rx.await?;
        match response {
            Response::LoginTokenRemoved { value } => Ok(value),
            _ => unreachable!("invalid response"),
        }
    }

    /// Store a new CreationToken.
    /// The token will be stored internally, and can be later removed by calling
    /// [`TokenStore::remove_creation_token`].
    pub async fn store_creation_token(
        &self,
        token: u32,
        account_id: u32,
        realm_id: u32,
    ) -> Result<(), crate::Error> {
        let (tx, rx) = oneshot::channel();
        let request = Request {
            payload: RequestPayload::StoreCreationToken {
                token,
                account_id,
                realm_id,
            },
            response: tx,
        };
        self.to_worker.send(request).await?;
        let response = rx.await?;
        match response {
            Response::Ok => Ok(()),
            _ => unreachable!("invalid response"),
        }
    }

    /// Remove a CreationToken.
    pub async fn remove_creation_token(
        &self,
        token: u32,
    ) -> Result<Option<CreationToken>, crate::Error> {
        let (tx, rx) = oneshot::channel();
        let request = Request {
            payload: RequestPayload::RemoveCreationToken { token },
            response: tx,
        };
        self.to_worker.send(request).await?;
        let response = rx.await?;
        match response {
            Response::CreationTokenRemoved { value } => Ok(value),
            _ => unreachable!("invalid response"),
        }
    }
}

impl Default for TokenStore {
    fn default() -> Self { Self::new() }
}

impl Drop for TokenStore {
    fn drop(&mut self) {
        // Before dropping the TokenStore, check if we are the last reference to
        // the worker handle. If we are, then we can abort the worker
        // task.
        if Arc::strong_count(&self.worker_handle) == 1 {
            self.worker_handle.abort();
        }
    }
}

impl TokenStoreWorker {
    async fn run(mut self) {
        while let Some(request) = self.from_controller.recv().await {
            let response = match request.payload {
                RequestPayload::GenerateLoginToken {
                    account_id,
                    realm_id,
                } => {
                    let token = rand::random();
                    self.login_tokens.insert(
                        token,
                        LoginToken {
                            account_id,
                            realm_id,
                        },
                    );
                    Response::LoginToken { token }
                },
                RequestPayload::RemoveLoginToken { token } => {
                    let value = self.login_tokens.remove(&token);
                    Response::LoginTokenRemoved { value }
                },
                RequestPayload::StoreCreationToken {
                    token,
                    account_id,
                    realm_id,
                } => {
                    self.creation_tokens.insert(
                        token,
                        CreationToken {
                            account_id,
                            realm_id,
                        },
                    );
                    Response::Ok
                },
                RequestPayload::RemoveCreationToken { token } => {
                    let value = self.creation_tokens.remove(&token);
                    Response::CreationTokenRemoved { value }
                },
            };
            let _ = request.response.send(response);
        }
    }
}
