# Worker-Distribution Roadmap

Notes on where this project is going, so we don't lose the ideas. Not all of this is implemented — this is the plan.

## Vision

A compile net over Raspberry Pis: a **distributer** (controller) schedules build tasks to **workers** that execute them and return results. Single codebase, one binary, roles decided at runtime.

## Architecture

- **Single binary, two roles** — decided by a CLI flag (`--role worker | distributer`):
  - Distributer: serves `/api/distribute`, `/register`, result endpoints; owns scheduling + worker validity.
  - Worker: serves `/api/work`; on startup receives the distributer's IP as a CLI arg and registers itself.
- Worker↔distributer and client↔distributer are independent links; they don't have to use the same protocol.

## Roadmap

- [X] Fix the handler ordering + shared `Arc` state (see above).
- [X] `--role` flag; worker accepts distributer IP via CLI arg (`std::env::args()` is enough; add `clap` only if flags multiply).
- [X] `POST /register` on the distributer; worker registers with its **own** IP + port on startup.
- [X] Execute real shell commands via `sh -c` with proper exit-code handling.
- [ ] Return a real result (stdout/stderr split, exit code, duration).
- [ ] Job model: `Uuid` job ID + status lifecycle (`queued → running → done/failed`).
- [ ] Client submits `POST /api/distribute` → gets back a job ID.
- [ ] Client polls `GET /result/<id>` (or `/api/result`) for status + output once finished.
- [ ] Job store on the distributer: `HashMap<Uuid, Job>` — will need `Mutex`/`RwLock` (interior mutability), same as the worker register.
- [ ] **Distributer-driven heartbeat**: distributer periodically pings each registered worker (`GET /health` on the worker) and marks them available/unavailable. The distributer is the single authority on which nodes are valid. Worker stays passive.
  - The heartbeat task mutates the same register the `/register` handler and scheduler touch — make sure the lock design covers all three access points.
- [ ] Timeouts + retries on worker communication.
- [ ] Load-aware scheduling: `is_available: bool` is too coarse. Track active job count (or CPU load) per worker so the distributer can oversubscribe correctly (a 4-core Pi can build more than one thing at once).
- [ ] Rewrite distribution as a **producer–consumer queue**: distributer pushes `Task`s into an `mpsc` channel; a pool of consumers pulls tasks and forwards them to workers. Decouples request handling from the actual forward.
  - This is also where the "pull" model becomes natural (workers/jobs drain from a queue instead of being pushed to a fixed URL).
  - Vocabulary: prefer `tokio::spawn` + `mpsc` (async tasks) over `std::thread`. Raw OS threads only if profiling shows the distributer is CPU-bound (unlikely here — compiling happens on workers). `tokio::spawn_blocking` for any genuinely blocking step (e.g. artifact sync).
- [ ] **Polling first** (`/result`), then add live output:
  - **SSE** (Server-Sent Events) for one-way output tailing — simpler than WS.
  - **WebSocket** only if bidirectional control is needed.
- [ ] **Cancellation**: stop a running process. On tokio that means owning the child process handle and being able to kill it (and the shell process tree) when a client requests cancel.
- [ ] If streaming the worker's output live: worker must stream stdout lines as they arrive (read line by line from the spawned process) rather than `.output().await` which buffers everything.
- [ ] Decide how built output gets back: worker writes to shared storage (rsync/scp/NFS), controller pulls over HTTP, or local-only for now.
  - This is usually the trickiest part of a compile farm — plan it early if real builds are the goal.

### Maker Faire demo
- [ ] **Event bus instead of synchronous hooks**: hooks (`before_distribute`, `on_worker_selected`, `before_work`, `after_work`, ...) don't draw directly. They emit `DisplayEvent`s into an `mpsc` channel; a dedicated display task consumes them and animates.
  - Hook call-sites map to: `distribute` handler (main.rs), `WorkerDistributor::distribute_task` (before/after the HTTP forward), `work` handler (before/after `ExecuteUtils::exec`).
  - `DisplayEvent` variants: `TaskReceived`, `WorkerSelected`, `WorkStarted`, `WorkFinished`, `WorkerRegistered`, plus an idle tick so the animation runs even between events.
  - Why: I2C/network draws are slow and must never block the request path; a dead display must not kill the service; the animation needs to keep running in idle state. Also the natural precursor to the channels/producer–consumer item above.
  - Display task is a single consumer that can feed **multiple renderers** — it's just another `mpsc` fan-out.
- [ ] **Renderer 1 — Browser dashboard** (laptop, also the client UI):
  - Laptop serves the web UI; visitors submit tasks from their own phones via a QR-code poster.
  - Show the live node map: 1 distributor + 5 Pis with status (idle/working/load), animated as tasks flow.
  - Optionally an SSE/WS feed (ties into the streaming item above).
- [ ] **Renderer 2 — SSD1306 OLED** on each Pi (distributor + workers):
  - Crates: `ssd1306` + `embedded-graphics` (teaches real trait/generic patterns), `linux-embedded-hal` for I2C. Alternative: `hd44780-driver` for a 16x2 char LCD.
  - Draw a mini "data center": 6 node dots (1 distributor + 5 workers); the distributor's screen highlights which node a task went to; each worker's screen animates while executing.
- [ ] Demo-day requirements (treat as scope, not nice-to-have):
  - **Wired switch, not event WiFi** — 5 Pis + visitors + wifi = dead demo.
  - Demo task: short, visible, repeatable (e.g. compile a tiny C program, or a script that simulates work with a progress bar). Real builds are too slow/risky.
  - Heartbeat/dead-node handling (the heartbeat item above) is required so a yanked Pi doesn't break the farm.

## Deployment notes

- **Cross-compilation is the main Pi friction**: you're on x86, the Pis are ARM. `rustup target add aarch64-unknown-linux-gnu` (+ matching toolchain, or musl). Do one experiment early.
- Heartbeat + re-registration matters because Pis reboot / lose power — a worker that registered once and is forgotten breaks the farm.

## Nice-to-haves / future ideas
- [ ] Worker health endpoint (`/health`) for the heartbeat.
- [ ] Job cancellation API.
- [ ] `clap` for CLI when flags multiply.
- [ ] Load metrics from workers (e.g. `nproc`, `/proc/loadavg`) for smarter scheduling.
- [ ] add load of worker instead of simply checking if its available or not
