# Authentication & Login — ✅ Implemented

The full 5017 login pipeline works: client → auth server → game server.

## Flow

1. Client connects to the auth server using the 5017 ciphers implemented in
   [crates/crypto](../../crates/crypto/) (TQ cipher, CQ cipher, RC5, plus a NOP
   cipher for tests).
2. `MsgConnect` (1052) and `MsgAccount` (1051) are handled by **WASM modules**
   ([packets/connect](../../packets/connect/src/lib.rs),
   [packets/account](../../packets/account/src/lib.rs)) executed in `wasmtime` by the
   auth `Runtime` ([server/auth/src/lib.rs](../../server/auth/src/lib.rs)).
3. `MsgAccount` authenticates against the `accounts` table (bcrypt, via
   [crates/db/src/account.rs](../../crates/db/src/account.rs)) and resolves the
   requested realm ([crates/db/src/realm.rs](../../crates/db/src/realm.rs)).
4. On success the host's server bus (`linker::server_bus::check` / `transfer`
   in [server/auth/src/linker.rs](../../server/auth/src/linker.rs)) verifies the
   game server is reachable and issues a login token; `MsgConnectEx`
   ([packets/connect-ex](../../packets/connect-ex/src/lib.rs)) returns either the
   game-server address or a rejection code.
5. The game server validates the token via `MsgTransfer`
   ([packets/transfer](../../packets/transfer/src/lib.rs),
   [server/game/src/packets/msg_transfer.rs](../../server/game/src/packets/msg_transfer.rs))
   and `MsgConnect`, then either enters the world or prompts character
   creation (`MsgTalk::login_new_role()`).

## Notable details

- The WASM modules expose `alloc` + `process_packet`; host functions cover
  logging, randomness, actor send/shutdown, DB auth, and the server bus.
- Integration tests cover both auth packets end-to-end against an in-memory
  SQLite database ([server/auth/src/lib.rs:134](../../server/auth/src/lib.rs#L134)).
- Seed accounts/realms come from migrations
  [6_default_accounts.sql](../../migrations/6_default_accounts.sql) and
  [7_default_realms.sql](../../migrations/7_default_realms.sql).

## Gaps

- No account registration flow (accounts must exist in DB).
- No ban/lockout, rate limiting, or session/duplicate-login management
  documented.
