# Worker-Distribution Roadmap

Notes on where this project is going, so we don't lose the ideas. Not all of this is implemented — this is the plan.

## Vision

A compile net over Raspberry Pis: a **distributer** (controller) schedules build tasks to **workers** that execute them and return results. Single codebase, one binary, roles decided at runtime.

## Architecture

- **Single binary, two roles** — decided by a CLI flag (`--role worker | distributer`):
  - Distributer: serves `/api/distribute`, `/register`, result endpoints; owns scheduling + worker validity.
  - Worker: serves `/api/work`; on startup receives the distributer's IP as a CLI arg and registers itself.
- Worker↔distributer and client↔distributer are independent links; they don't have to use the same protocol.

## Current known bugs / first fixes

- [ ] **Handler extractor order** (main.rs `distribute`): `State` (a `FromRequestParts` extractor) must come **before** `Json` (a body `FromRequest` extractor). A body extractor may only be the *last* argument, and only the last argument may be a body extractor. This is why the handler never satisfied `Handler`.
- [ ] **Use the shared state**: the `Arc<WorkerDistributer>` stored via `.with_state` is currently shadowed and a fresh `WorkerDistributer` is rebuilt per request. Extract and use the `Arc` from `State<Arc<...>>`.
- [ ] **Build the worker URL properly** (`worker_distributer.rs`): `format!("{}/api/work", worker.ip)` produces `localhost/api/work` — missing scheme (`http://`) and port. Worker registration should provide both so the distributer can construct the real URL.
- [ ] **Actually execute commands** (`execute_utils.rs`): currently runs `echo`, so nothing executes. Use `sh -c "<script>"` so the shell parses pipes/redirection/`&&`. Prefer `tokio::process::Command` (async) so long builds don't block the runtime, and capture `output.status` (exit code) instead of `.expect`.
  - Note: `sh -c` with user-supplied input is shell injection — acceptable on a trusted LAN, but be aware of it.

## Roadmap

### Phase 1 — Make the happy path real
- [ ] Fix the handler ordering + shared `Arc` state (see above).
- [ ] `--role` flag; worker accepts distributer IP via CLI arg (`std::env::args()` is enough; add `clap` only if flags multiply).
- [ ] `POST /register` on the distributer; worker registers with its **own** IP + port on startup.
- [ ] Execute real shell commands via `sh -c` with proper exit-code handling.
- [ ] Return a real result (stdout/stderr split, exit code, duration).

### Phase 2 — Job semantics
- [ ] Job model: `Uuid` job ID + status lifecycle (`queued → running → done/failed`).
- [ ] Client submits `POST /api/distribute` → gets back a job ID.
- [ ] Client polls `GET /result/<id>` (or `/api/result`) for status + output once finished.
- [ ] Job store on the distributer: `HashMap<Uuid, Job>` — will need `Mutex`/`RwLock` (interior mutability), same as the worker register.

### Phase 3 — Resilience / scheduling
- [ ] **Distributer-driven heartbeat**: distributer periodically pings each registered worker (`GET /health` on the worker) and marks them available/unavailable. The distributer is the single authority on which nodes are valid. Worker stays passive.
  - The heartbeat task mutates the same register the `/register` handler and scheduler touch — make sure the lock design covers all three access points.
- [ ] Timeouts + retries on worker communication.
- [ ] Load-aware scheduling: `is_available: bool` is too coarse. Track active job count (or CPU load) per worker so the distributer can oversubscribe correctly (a 4-core Pi can build more than one thing at once).

### Phase 4 — Distribution via channels
- [ ] Rewrite distribution as a **producer–consumer queue**: distributer pushes `Task`s into an `mpsc` channel; a pool of consumers pulls tasks and forwards them to workers. Decouples request handling from the actual forward.
- [ ] This is also where the "pull" model becomes natural (workers/jobs drain from a queue instead of being pushed to a fixed URL).
- [ ] Vocabulary: prefer `tokio::spawn` + `mpsc` (async tasks) over `std::thread`. Raw OS threads only if profiling shows the distributer is CPU-bound (unlikely here — compiling happens on workers). `tokio::spawn_blocking` for any genuinely blocking step (e.g. artifact sync).

### Phase 5 — Results & streaming (post-Phase 2)
- [ ] Job store is the hard 80% — streaming is a thin layer on top, not a rewrite.
- [ ] **Polling first** (`/result`), then add live output:
  - **SSE** (Server-Sent Events) for one-way output tailing — simpler than WS.
  - **WebSocket** only if bidirectional control is needed.
- [ ] **Cancellation**: stop a running process. On tokio that means owning the child process handle and being able to kill it (and the shell process tree) when a client requests cancel.
- [ ] If streaming the worker's output live: worker must stream stdout lines as they arrive (read line by line from the spawned process) rather than `.output().await` which buffers everything.

### Phase 6 — Artifacts (still undecided)
- [ ] Decide how built output gets back: worker writes to shared storage (rsync/scp/NFS), controller pulls over HTTP, or local-only for now.
- [ ] This is usually the trickiest part of a compile farm — plan it early if real builds are the goal.

## Deployment notes

- **Cross-compilation is the main Pi friction**: you're on x86, the Pis are ARM. `rustup target add aarch64-unknown-linux-gnu` (+ matching toolchain, or musl). Do one experiment early.
- Heartbeat + re-registration matters because Pis reboot / lose power — a worker that registered once and is forgotten breaks the farm.

## Nice-to-haves / future ideas
- [ ] Worker health endpoint (`/health`) for the heartbeat.
- [ ] Job cancellation API.
- [ ] `clap` for CLI when flags multiply.
- [ ] Load metrics from workers (e.g. `nproc`, `/proc/loadavg`) for smarter scheduling.
- [ ] add load of worker instead of simply checking if its available or not
