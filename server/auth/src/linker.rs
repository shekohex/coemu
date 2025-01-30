use wasmtime::Linker;

pub const MODULE: &str = "host";
pub const ALLOC: &str = "__alloc";
pub const MEMORY: &str = "memory";

pub fn encode_ptr_len(a: i32, b: usize) -> u64 {
    (a as u64) << 32 | b as u64
}

pub fn decode_ptr_len(c: u64) -> (i32, usize) {
    ((c >> 32) as u32 as i32, c as u32 as usize)
}

macro_rules! memof {
    ($caller:expr) => {
        $caller
            .get_export($crate::linker::MEMORY)
            .and_then(::wasmtime::Extern::into_memory)
            .expect("failed to read wasm memory")
    };
}

macro_rules! mread {
    ($caller:expr, $mem:expr, $ptr:expr, $size:expr) => {{
        $mem.data(&$caller)
            .get($ptr as usize..)
            .and_then(|s| s.get(..$size as usize))
            .expect("failed to read wasm memory")
    }};
}

macro_rules! mread_mut {
    ($caller:expr, $mem:expr, $ptr:expr, $size:expr) => {{
        $mem.data_mut(&mut $caller)
            .get_mut($ptr as usize..)
            .and_then(|s| s.get_mut(..$size as usize))
            .expect("failed to read wasm memory")
    }};
}

macro_rules! alloc {
    ($caller:expr) => {{
        $caller
            .get_export($crate::linker::ALLOC)
            .and_then(::wasmtime::Extern::into_func)
            .expect("failed to read wasm alloc function")
            .typed::<u32, i32>(&mut $caller)
            .expect("failed to read wasm typed alloc function")
    }};
}

pub mod network {
    pub mod actor {
        use wasmtime::{ExternRef, Linker};

        use crate::linker::MODULE;

        pub fn set_id(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_network_actor_set_id";

            linker.func_wrap2_async::<Option<ExternRef>, i32, ()>(MODULE, NAME, |_caller, actor_ref, id| {
                Box::new(async move {
                    let actor_ref = actor_ref.expect("actor ref not null");
                    let actor = actor_ref
                        .data()
                        .downcast_ref::<tq_network::ActorHandle>()
                        .expect("actor ref is valid");
                    actor.set_id(id as usize);
                }) as _
            })?;
            Ok(())
        }

        pub fn shutdown(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_network_actor_shutdown";

            linker.func_wrap1_async::<Option<ExternRef>, ()>(MODULE, NAME, |_caller, actor_ref| {
                Box::new(async move {
                    let actor_ref = actor_ref.expect("actor ref not null");
                    let actor = actor_ref
                        .data()
                        .downcast_ref::<tq_network::ActorHandle>()
                        .expect("actor ref is valid");
                    let _ = actor.shutdown().await;
                }) as _
            })?;
            Ok(())
        }

        pub fn send(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_network_actor_send";
            linker.func_wrap3_async::<Option<ExternRef>, u32, u64, ()>(
                MODULE,
                NAME,
                |mut caller, actor_ref, packet_id, packet_data| {
                    Box::new(async move {
                        let actor_ref = actor_ref.expect("actor ref not null");
                        let actor = actor_ref
                            .data()
                            .downcast_ref::<tq_network::ActorHandle>()
                            .expect("actor ref is valid");
                        let mem = memof!(caller);
                        let (packet_data_ptr, packet_data_len) = crate::linker::decode_ptr_len(packet_data);
                        let packet_data = mread!(caller, mem, packet_data_ptr, packet_data_len);
                        let _ = actor.send((packet_id as u16, packet_data)).await;
                    }) as _
                },
            )?;
            Ok(())
        }
    }
}

pub mod rand {
    use wasmtime::Linker;

    use crate::linker::MODULE;
    use rand::Rng;

    pub fn getrandom(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
        const NAME: &str = "getrandom";
        linker.func_wrap1_async::<u64, i32>(MODULE, NAME, |mut caller, buffer| {
            Box::new(async move {
                let mem = memof!(caller);
                let (ptr, len) = crate::linker::decode_ptr_len(buffer);
                let slice = mread_mut!(caller, mem, ptr, len);
                let mut rng = rand::thread_rng();
                rng.fill(slice);
                0
            }) as _
        })?;
        Ok(())
    }
}

pub mod log {
    use super::*;
    pub fn trace_event(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
        const NAME: &str = "trace_event";
        linker.func_wrap5_async::<i32, i32, i32, i32, i32, ()>(
            MODULE,
            NAME,
            |mut caller, level, target, target_len, message, message_len| {
                Box::new(async move {
                    let mem = memof!(caller);
                    let target_slice = mread!(caller, mem, target, target_len);
                    let target = std::str::from_utf8(target_slice).expect("valid utf8");
                    let message_slice = mread!(caller, mem, message, message_len);
                    let message = std::str::from_utf8(message_slice).expect("valid utf8");
                    match level {
                        0 => tracing::error!(target: "runtime", packet = target, %message),
                        1 => tracing::warn!(target: "runtime", packet = target, %message),
                        2 => tracing::info!(target: "runtime", packet = target, %message),
                        3 => tracing::debug!(target: "runtime", packet = target, %message),
                        _ => tracing::trace!(target: "runtime", packet = target, %message),
                    };
                }) as _
            },
        )?;
        Ok(())
    }
}

pub mod db {
    pub mod account {
        use wasmtime::Linker;

        use crate::linker::MODULE;

        pub fn auth(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_db_account_auth";
            linker.func_wrap2_async::<u64, u64, i32>(MODULE, NAME, |mut caller, username, password| {
                Box::new(async move {
                    let mem = memof!(caller);
                    let (username_ptr, username_len) = crate::linker::decode_ptr_len(username);
                    let username_slice = mread!(caller, mem, username_ptr, username_len);
                    let username = std::str::from_utf8(username_slice).expect("valid utf8");
                    let (password_ptr, password_len) = crate::linker::decode_ptr_len(password);
                    let password_slice = mread!(caller, mem, password_ptr, password_len);
                    let password = std::str::from_utf8(password_slice).expect("valid utf8");
                    let pool = caller.data().pool();
                    let account = tq_db::account::Account::auth(pool, username, password).await;
                    match account {
                        Ok(account) => account.account_id,
                        Err(tq_db::Error::AccountNotFound) => -1,
                        Err(tq_db::Error::InvalidPassword) => -2,
                        Err(e) => {
                            tracing::error!("Failed to auth account: {}", e);
                            -1
                        },
                    }
                }) as _
            })?;
            Ok(())
        }
    }

    pub mod realm {
        use wasmtime::Linker;

        use crate::linker::MODULE;

        pub fn by_name(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_db_realm_by_name";
            linker.func_wrap1_async::<u64, u64>(MODULE, NAME, |mut caller, name| {
                Box::new(async move {
                    let mem = memof!(caller);
                    let (name_ptr, name_len) = crate::linker::decode_ptr_len(name);
                    let name_slice = mread!(caller, mem, name_ptr, name_len);
                    let name = std::str::from_utf8(name_slice).expect("valid utf8");
                    let pool = caller.data().pool();
                    let realm = tq_db::realm::Realm::by_name(pool, name).await;
                    let archived = match realm {
                        Ok(Some(realm)) => rkyv::to_bytes::<_, 64>(&realm).expect("failed to archive realm"),
                        Ok(None) => return 0,
                        Err(e) => {
                            tracing::error!("Failed to get realm by name: {e}",);
                            return 0;
                        },
                    };
                    let alloc = alloc!(caller);
                    let ptr = alloc
                        .call_async(&mut caller, archived.len() as u32)
                        .await
                        .expect("failed to allocate memory");
                    mem.write(&mut caller, ptr as usize, &archived)
                        .expect("failed to write realm to memory");
                    crate::linker::encode_ptr_len(ptr, archived.len())
                }) as _
            })?;
            Ok(())
        }
    }
}

pub mod server_bus {
    use msg_transfer::MsgTransfer;
    use tokio::net::TcpStream;
    use tokio_stream::StreamExt;
    use tq_network::{CQCipher, PacketDecode, PacketEncode, PacketID, TQCodec};
    use tracing::Instrument;
    use wasmtime::{ExternRef, Linker};

    use crate::linker::MODULE;

    pub fn check(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
        const NAME: &str = "auth_server_bus_check";
        linker.func_wrap1_async::<u64, i32>(MODULE, NAME, |mut caller, realm| {
            Box::new(async move {
                let mem = memof!(caller);
                let (realm_ptr, realm_len) = crate::linker::decode_ptr_len(realm);
                let realm_slice = mread!(caller, mem, realm_ptr, realm_len);
                let realm = unsafe {
                    rkyv::from_bytes_unchecked::<tq_db::realm::Realm>(realm_slice).expect("failed to deserialize realm")
                };
                let ip = realm.game_ip_address.as_str();
                let port = realm.game_port;
                let stream = TcpStream::connect(format!("{ip}:{port}"))
                    .instrument(tracing::info_span!("realm_connect", %ip, %port, realm_id = realm.realm_id))
                    .await;
                match stream {
                    Ok(_) => 0,
                    Err(e) => {
                        tracing::error!(
                            %ip,
                            %port,
                            realm_id = realm.realm_id,
                            error = ?e,
                            "Failed to connect to realm"
                        );
                        -1
                    },
                }
            }) as _
        })?;
        Ok(())
    }

    pub fn transfer(linker: &mut Linker<crate::State>) -> Result<(), crate::error::Error> {
        const NAME: &str = "auth_server_bus_transfer";
        linker.func_wrap2_async::<Option<ExternRef>, u64, i64>(MODULE, NAME, |mut caller, actor_ref, realm| {
            Box::new(async move {
                let actor_ref = actor_ref.expect("actor ref not null");
                let actor = actor_ref
                    .data()
                    .downcast_ref::<tq_network::ActorHandle>()
                    .expect("actor ref is valid");
                let mem = memof!(caller);
                let (realm_ptr, realm_len) = crate::linker::decode_ptr_len(realm);
                let realm_slice = mread!(caller, mem, realm_ptr, realm_len);
                let realm = unsafe {
                    rkyv::from_bytes_unchecked::<tq_db::realm::Realm>(realm_slice).expect("failed to deserialize realm")
                };
                let ip = realm.game_ip_address.as_str();
                let port = realm.game_port;
                let stream = TcpStream::connect(format!("{ip}:{port}"))
                    .instrument(tracing::info_span!("realm_connect", %ip, %port, realm_id = realm.realm_id))
                    .await;
                let tcp_stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!(
                            %ip,
                            %port,
                            realm_id = realm.realm_id,
                            error = ?e,
                            "Failed to connect to realm"
                        );
                        return -1;
                    },
                };
                let cipher = CQCipher::new();
                let (mut encoder, mut decoder) = TQCodec::new(tcp_stream, cipher).split();
                let transfer = MsgTransfer {
                    account_id: actor.id() as _,
                    realm_id: realm.realm_id as _,
                    ..Default::default()
                };

                let transfer = transfer.encode().expect("failed to encode transfer");
                encoder.send(transfer).await.expect("failed to send transfer");
                let res = decoder.next().await;
                let res = match res {
                    Some(Ok((MsgTransfer::PACKET_ID, bytes))) => {
                        MsgTransfer::decode(&bytes).expect("failed to decode transfer")
                    },
                    Some(Ok((id, _))) => {
                        tracing::error!(packet_id = ?id, "Unexpected packet id");
                        return -1;
                    },
                    Some(Err(e)) => {
                        tracing::error!(error = ?e, "Failed to decode transfer");
                        return -1;
                    },
                    None => {
                        return -1;
                    },
                };
                res.token as i64
            }) as _
        })?;
        Ok(())
    }
}
