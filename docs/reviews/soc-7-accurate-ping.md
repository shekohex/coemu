# Review: SOC-7 — Accurate ping (verdict: APPROVE)

Reviewer: rust-mentor-reviewer · Date: 2026-06-12 · Change set: uncommitted
working tree on `story/soc-7-accurate-ping`

Files in the diff:

- `server/game/src/packets/msg_item.rs` — the fix + five tests
- `server/game/src/test_utils.rs` — `init()` → `try_init()` for the tracing
  subscriber
- `crates/crypto/src/rc5.rs` — test typo fix (`origional` → `original`)

> Verification note: no Rust toolchain is installed on this machine, so this
> review is a static read of the diff against the surrounding code
> (`tq-network`, `tq-serde`, the `PacketHandler` derive). QA ran the suite and
> reported green; nothing I traced contradicts that.

## Summary

The story ([SOC-7](../user-story/social.md)) asks for a truthful latency
display: the `Ping` arm of the `MsgItem` (1009) handler used to rebuild the
packet with `client_timestamp + 30` before echoing it — an admitted hack that
made the client's computed round-trip time wrong (sometimes negative). The fix
deletes the rebuilt packet and the fudge entirely; the ping arm now echoes the
received packet verbatim with `actor.send(self.clone())`, exactly like the
fall-through arm already did (`msg_item.rs:72-77`). This matches both the
acceptance criteria and the protocol notes
([co-5017-protocol-notes.md:60](../notes/co-5017-protocol-notes.md): PING is a
client→server request; the client owns the timestamp math).

A side effect worth celebrating: the old `self.client_timestamp + 30` was a
**latent crash**. In debug builds, `u32` addition panics on overflow, and the
timestamp is entirely client-controlled — any client sending a value above
`u32::MAX - 30` (deliberately, or innocently after ~49.7 days of machine
uptime, since the client timestamp is a `GetTickCount`-style millisecond
counter that wraps) would panic the packet task. The fix removes a
network-reachable panic, not just a cosmetic skew. See lesson 1 below.

Tests: one happy-path echo test (implementer) plus four adversarial ones (QA):
boundary timestamps (`0`, `29`, `30`, `u32::MAX - 30`, `u32::MAX`), bit-exact
echo of all five fields with an exactly-one-packet assertion, a regression
guard that unknown actions still take the diagnostic arm, and a
truncated-body decode test. All five check out against the actual
`tq-network`/`tq-serde` behavior I traced (e.g. `tq_serde` deserialization
returns `TQSerdeError::Eof` on short input — `crates/serde/src/de.rs:25` — so
the no-panic claim holds).

The two collateral fixes are correct and necessary:

- `test_utils.rs:39`: `with_test_env` was previously called from only one test
  module (`world/map.rs:582`), so the panicking `init()` never collided. The
  new tests in `msg_item.rs` share the same test binary, and tests run
  concurrently — the second `init()` would panic. `try_init()` with `let _ =`
  is the standard idiom.
- `rc5.rs:223`: before this fix the `encrypt_decrypt` test referenced an
  undefined variable `origional`, meaning **the crypto crate's test target did
  not compile at all**. Worth noting that this went unnoticed — see question 3.

## Findings

No blocking findings.

### 🟡 should-fix

1. **Truncated-decode test contradicts the protocol notes it cites** —
   `msg_item.rs:255-262` vs
   [co-5017-protocol-notes.md:33-34](../notes/co-5017-protocol-notes.md).
   The test's doc comment says "5017 MsgItem requests legitimately arrive in
   multiple sizes", and the protocol notes agree ("Requests may arrive
   truncated — don't read unspecified trailing fields"). But the test then
   asserts every body shorter than 20 bytes **must fail to decode** — and on
   decode failure the generated handler logs and silently drops the packet
   (`macros/derive-packethandler/src/lib.rs:104-107`). Both statements can't
   be true: either short bodies are legitimate (then dropping them is a latent
   compatibility gap and the test enshrines the wrong behavior), or the 5017
   client always sends the full 20-byte body for the actions we handle (then
   the comment and the protocol note overclaim). The asserted behavior is
   fine as a *no-panic guard* — but the comment should not document the
   opposite of what the test enforces. Suggested rewording:

   ```rust
   /// QA (SOC-7 hostile input): a truncated MsgItem body must fail decoding
   /// with a clean error, never a panic. Note: today the handler drops
   /// undecodable packets (derive-packethandler logs and returns Ok); if the
   /// protocol notes are right that short 1009 bodies legitimately occur,
   /// optional trailing fields are a follow-up story, not a decode error.
   ```

   Not blocking because the runtime behavior is unchanged by this diff and a
   dropped malformed ping costs the client one missed echo, nothing more.

### 🟢 nit

2. **"Hostile/unknown" mislabels a known action** — `msg_item.rs:213-220`.
   In `unknown_action_still_sends_missing_action_diagnostic`, the probe value
   `7` is `ItemActionType::SplitItem` — a *known* variant that is merely
   unimplemented, unlike `0`, `30`, `0xDEAD_BEEF`, `u32::MAX` which all map to
   `Unknown`. The assertion is still correct (every non-`Ping` action takes
   the `_` arm), and covering a known-but-unimplemented action is actually
   valuable — but the comment should say so instead of calling all five
   "hostile/unknown".

3. **Duplicated echo across match arms** — `msg_item.rs:76` and `:79`. Both
   arms begin with `actor.send(self.clone()).await?;`. It could be hoisted
   above the `match`, leaving the `match` to handle only the diagnostic:

   ```rust
   actor.send(self.clone()).await?;
   if !matches!(action, ItemActionType::Ping) {
       // ... MsgTalk diagnostic + warn ...
   }
   ```

   Counterpoint: the explicit `Ping` arm with its comment documents protocol
   intent, and future real actions (Buy, Sell, …) will *not* start with an
   echo, at which point the hoist would have to be unwound. Keeping the
   duplication is a defensible choice — flagging only so it's a choice, not
   an accident.

4. **Test boilerplate** — the recv-decode-assert block is repeated three times
   (`msg_item.rs:121-127`, `:156-162`, `:189-196`). A small helper in the
   tests module, e.g.
   `fn recv_packet<P: PacketDecode>(rx: &mut Receiver<Message>) -> P`,
   would shrink each test and make the assertions the only thing that varies.
   Fine to leave for now; worth doing the next time a packet test is added.

5. **`MsgItem` could be `Copy`** — `msg_item.rs:53-61`. The struct is five
   `u32`s (20 bytes, no heap data). Adding `Copy` to the derive list would let
   the handler write `actor.send(*self)` and make the "this is a trivially
   cheap duplication" property visible in the type. Purely optional — see
   lesson 2 for why `clone()` is needed at all.

## Rust lessons from this diff

Four concepts, anchored to TypeScript. This is the first review in
`docs/reviews/`, so these start from zero.

### 1. The bug that was deleted: integer overflow is a *semantics* decision

The line this story kills:

```rust
client_timestamp: self.client_timestamp + 30,
```

In TypeScript this is boring: `number` is an IEEE-754 double, so
`4294967295 + 30` is just `4294967325` — wrong for the protocol (it no longer
fits the wire's 4 bytes) but never a crash. Rust's `u32` is an actual 32-bit
unsigned integer, and `u32::MAX + 30` *cannot be represented*. What happens
next depends on build profile:

- **Debug builds** (`cargo build`, `cargo test`): overflow **panics** —
  `attempt to add with overflow`. The thread running the packet handler dies.
- **Release builds** (`cargo build --release`): overflow **wraps** silently
  (`u32::MAX + 30` → `29`), like C. The panic check is compiled out for speed.

So the old code was a packet-of-death in debug and a silent skew in release —
the worst combination, because tests crash where production corrupts. And the
input is fully attacker-controlled: `client_timestamp` comes straight off the
socket. This is exactly the "panics reachable from network input" class this
project's review checklist exists for.

When you *intend* arithmetic on untrusted or wrap-prone numbers, Rust makes
you pick the semantics explicitly:

```rust
x.wrapping_add(30)    // modular arithmetic, what GetTickCount math wants
x.checked_add(30)     // -> Option<u32>, None on overflow (like T | undefined)
x.saturating_add(30)  // clamps at u32::MAX
```

The fix here is better than all of these: don't do the math at all. But file
the rule away — **any `+`/`-`/`*` on a client-supplied integer needs a reason
it can't overflow, or an explicit `wrapping_`/`checked_` spelling.** It will
come up again immediately on the money paths (`SaveMoney`, `DrawMoney` in
this very enum).

### 2. `actor.send(self.clone())` — why the clone, and where the value goes

The whole fix is one line, and it's the most Rust-dense line in the diff:

```rust
async fn process(&self, _state: &Self::State, actor: &Actor<Self::ActorState>) -> ... {
    ...
    actor.send(self.clone()).await?;
```

In TypeScript, every object is a GC'd reference; passing `this` to
`actor.send(this)` shares it, and nobody asks who owns it afterwards. Rust
asks. Two facts collide here:

1. `process` receives `&self` — a **borrow** (a temporary, read-only view; the
   caller still owns the `MsgItem`). You may read through it, but you may not
   give the `MsgItem` itself away.
2. `Actor::send` takes its argument **by value**
   (`crates/network/src/actor.rs:110`):

   ```rust
   pub async fn send<P: PacketEncode>(&self, packet: P) -> Result<(), P::Error>
   ```

   `packet: P` (no `&`) means the value is **moved** into `send` — ownership
   transfers, and the caller can't use it afterwards. That's a concept with no
   TS equivalent: in TS, "I passed it to you" and "I can still use it" are
   both always true.

You cannot move what you only borrowed — writing `actor.send(*self)` would be
"give away the thing behind a read-only view", and the compiler rejects it
(`cannot move out of *self which is behind a shared reference`). So you mint a
fresh, owned duplicate: `self.clone()`. For this struct that's a 20-byte
memcpy — five `u32`s — effectively free (hence finding 5: it could be `Copy`,
making the duplication implicit and provably trivial).

Worth tracing one level deeper, because it explains the test harness: `send`
doesn't push your struct through the channel. It serializes first
(`actor.rs:147-151`):

```rust
let msg = packet.encode()?;                      // -> (u16, Bytes)
self.tx.send(msg.into()).map_err(Into::into).await?;  // Message::Packet(id, bytes)
```

The actor's mailbox is a `tokio::sync::mpsc::Sender<Message>` — a **bounded**
channel, the closest thing Rust has to "the event loop queue", except
explicit: when the queue is full, `.send(...).await` *suspends the sender*
(backpressure) instead of growing an unbounded heap like a busy Node process
would. That's why the tests can build an `Actor` from a bare
`tokio::sync::mpsc::channel(1)` and then assert on `rx.try_recv()`: the
"network" in tests is just the receiving end of the mailbox, and what comes
out is the actual wire encoding (`Message::Packet(1009, 20 bytes)`), which the
tests then `MsgItem::decode` back. The tests exercise the real
serialize→channel→deserialize path, not a mock.

### 3. `match` on a `num_enum` — discriminated unions where the compiler does the narrowing

The dispatch at the top of the handler (`msg_item.rs:70-71`):

```rust
let action = self.action_type.into();
match action {
    ItemActionType::Ping => { ... },
    _ => { ... },
}
```

Two things here look like magic until you see the machinery.

**Where does `.into()` know its target type?** Type inference flows
*backwards* from the `match`: the arms pattern-match `ItemActionType`
variants, so `action` must be `ItemActionType`, so `.into()` resolves to
`impl From<u32> for ItemActionType`. TS infers in the same direction
sometimes (contextual typing), but it would never pick *which conversion
function to call* from how the result is later used.

**Where does `From<u32>` come from?** The derive on the enum
(`msg_item.rs:13-17`):

```rust
#[derive(Default, Debug, FromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u32)]
enum ItemActionType {
    #[default]
    Unknown,
    Buy = 1,
    ...
```

`#[derive(...)]` looks like a TS decorator but is compile-time code
generation, not runtime metadata: `num_enum`'s `FromPrimitive` macro writes
the `From<u32>` impl for you, and `#[default] Unknown` makes it **total** —
any `u32` that matches no discriminant becomes `Unknown` instead of failing.
Compare TS: `const action: ItemActionType = packet.actionType` does *no*
checking at all (any number passes the type checker), and a reverse enum
lookup gives you `undefined`. Rust forces the unknown case to be a real,
named state you must handle. That's why QA's hostile-action test
(`0xDEAD_BEEF`, `u32::MAX`, …) can assert a defined behavior — there is no
"undefined" path through this function.

And the property you'll lean on constantly: `match` is
**exhaustiveness-checked**, like a `switch` over a TS discriminated union
with `never`-checking — except built in and non-optional. Today the `_` arm
swallows everything except `Ping`. The day `Buy` gets a real implementation
the temptation will be to add it above `_`; at the point where the `_` arm is
finally removed, the compiler will list every variant you forgot. (TS's
closest analog, the `default: assertNever(x)` trick, is a convention; this is
the language.)

### 4. `#[async_trait]` and associated types — why the trait impl looks heavier than a TS interface

The handler implements this (`msg_item.rs:63-67`):

```rust
#[async_trait]
impl PacketProcess for MsgItem {
    type ActorState = ActorState;
    type Error = crate::Error;
    type State = State;

    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>)
        -> Result<(), Self::Error> { ... }
}
```

In TS this whole shape is one line:
`interface PacketProcess { process(state: State, actor: Actor): Promise<void> }`.
Two gaps between the languages explain the extra ceremony:

**Associated types** (`type Error = crate::Error;`) are outputs of the trait
implementation, not inputs from the caller. The TS-generics instinct would be
`interface PacketProcess<S, E>` — but then every *user* of the trait picks the
types, and one packet could implement it twice with different states. An
associated type says: *for `MsgItem`, there is exactly one `State`, one
`Error`* — the implementer decides, the caller just writes `Self::Error`.
You can see the payoff in the generated dispatcher
(`macros/derive-packethandler/src/lib.rs:97-109`): it calls
`msg.process(state, actor).await?` and the `?` works because every packet in
the `Handler` enum agreed on the same `Error` type.

**`#[async_trait]`**: TS interfaces get async methods for free because every
function returns a `Promise` — a heap reference, uniform size, eagerly
started. A Rust `async fn` compiles to a *state machine type unique to that
function*, whose size must be known and which does nothing until polled
("lazy futures" — the `.await` is where execution actually advances, unlike a
Promise which is already running). Traits with differently-sized return types
per implementor didn't work when this code was written, so the
`#[async_trait]` macro rewrites the method to return a boxed future
(`Pin<Box<dyn Future<...> + Send>>`) — roughly "heap-allocate the state
machine so every implementor's return value is one pointer big". That's the
most Promise-like thing in Rust, bought with one allocation per call. (Since
Rust 1.75 native `async fn` in traits exists and this crate could migrate
some day, but `#[async_trait]` is still the standard answer when the trait
must be object-safe, as `PacketHandler`-style dispatch wants.)

One bonus from the `Send` in that boxed future: it's the compiler-checked
promise that the future may hop between tokio's worker threads. Node never
asks because there's one event loop thread; Rust asks at compile time, which
is exactly why `Arc` (atomically reference-counted shared ownership, used by
`ActorHandle` at `actor.rs:37`) shows up where a TS service would just close
over a variable.

## Questions for the author

1. **Truncated 1009 bodies in the wild** (ties to finding 1): have you
   observed the real 5017 client sending short `MsgItem` bodies — for ping or
   any action? If yes, the silent log-and-drop in the generated handler means
   those actions no-op today and the protocol note deserves its own story; if
   no for ping specifically, the note's "requests may arrive truncated" could
   be annotated with *which* actions, and the test comment tightened.

2. **Echoing non-ping actions** (`msg_item.rs:79`, pre-existing): the `_` arm
   echoes the request back before the "Missing Item Action Type" diagnostic.
   Do we know the 5017 client treats an echoed `Buy`/`Sell`/`SaveMoney` as
   inert, rather than as a server-confirmed action? An optimistic client
   could interpret the echo as success (the wiki's anti-dupe warnings make me
   twitchy around the money actions). Out of scope for SOC-7, but if it's
   unverified it may deserve a story before any item actions ship.

3. **CI gap revealed by the `origional` fix**: the crypto crate's tests
   didn't compile before this branch, which suggests `cargo test --workspace`
   isn't running (or isn't gating) on CI. Is that worth a chore story? This
   diff's new tests only pay rent if something runs them.

4. **Test channel capacities**: the tests use `channel(1)` and `channel(8)`
   ad hoc. Capacity 1 works today because ping sends exactly one packet — if
   a future change made the handler send two, the test would deadlock-ish
   (fail on a full channel) rather than fail with a clear assertion. Was the
   asymmetry deliberate, or worth standardizing on a roomier default in a
   shared helper (finding 4)?
