# oh-my-docker (omdocker)

A keyboard-driven Docker TUI (Terminal User Interface) written in Rust.

Fast, minimal, and built for ops workflows — inspired by k9s, lazygit, and htop.

## View in action

[<img src="https://img.youtube.com/vi/l9wLgjm5CNo/hqdefault.jpg" width="600" height="300"
/>](https://www.youtube.com/embed/l9wLgjm5CNo)

## Quick Start

Download binary from [github releases](https://github.com/supertorpe/oh-my-docker/releases)

...or build it yourself:
```bash
cargo build --release
```
...or build it yourself with Docker:
```bash
mkdir -p .cache && docker run --rm \
	-u $(id -u):$(id -g) \
	-v $PWD:/volume \
	-v $PWD/.cache:/usr/local/cargo/registry \
	-w /volume \
	rust:slim-bookworm \
	cargo build --release
```
Run:
```bash
./target/release/omdocker
```

Requires Docker to be installed and the user to have access to the Docker socket.

## Features

- **Container list** — browse running/stopped containers with fuzzy search, optional initial filter via CLI arg
- **Container details** — inspect metadata, env, volumes, networks, ports, labels; scrollable with `j`/`k`/`PgUp`/`PgDn`
- **Multi-select mode** — press `Space` to enter selection mode, select individual containers, batch start/stop/delete
- **Live logs** — streaming with follow mode, pause, search, scroll, jump-to-top/bottom, timestamp toggle (`T`), export to file (`Ctrl+S`)
- **Shell access** — `docker exec -it` inside any container with configurable shell (`sh`, `bash`, `/bin/zsh`), user (container default, `root`, `host` → `uid:gid`, or custom `user:group`), and working directory. Config per container is persisted to `~/.config/omdocker/omdocker.toml`. TUI suspends, shell runs in the parent terminal, TUI resumes on exit.
- **Image management** — list, remove, run containers from images with configurable command/env/ports/volumes/name/auto-remove and inline validation
- **Dangling/prune images** — remove dangling (`<none>`) images with `D`, prune all unused images with `p`
- **Docker events** — real-time event stream with type filtering, scroll, export, pause, and emoji icons
- **Statistics** — live `docker stats` view with CPU %, memory, network I/O, block I/O, PIDs; sortable by column with `←`/`→`, toggle direction with `t`
- **Networks** — list and delete Docker networks
- **Volumes** — list and delete Docker volumes
- **Container lifecycle** — start, stop (shows "stopping..."), restart, delete (shows "deleting...") with confirmation dialogs
- **Docker reconnection** — automatic periodic reconnection with visual feedback when Docker is unavailable
- **Persistent errors** — red error toasts for critical messages, dismiss with any key; auto-dismissing info toasts
- **Self-update** — background check for new versions on startup (configurable), `U` to check/download, auto-replaces binary
- **Keyboard-first** — all actions available via keys, no mouse needed
- **Fast** — async polling, non-blocking UI, ring buffers for logs/events
- **CLI** — `--help`/`-h` and `--version`/`-V` flags

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Toggle help |
| `Esc` | Go back / close search / close form |
| `U` | Check for updates / download available update |

### Containers
| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `Enter` | Open container details |
| `/` | Activate fuzzy search |
| `l` | Open logs |
| `s` | Open shell (`docker exec -it`) |
| `t` | Start/stop container (shows "stopping..." while stopping) |
| `r` | Restart container |
| `d` | Delete container (shows "deleting...") with confirmation |
| `Space` | Toggle selection mode; in selection mode, toggle single container |
| `Ctrl+A` | Select all filtered containers (in selection mode) |
| `Esc` | Exit selection mode |
| `i` | Switch to images view |
| `e` | Switch to events view |
| `%` | Switch to statistics view |
| `n` | Switch to networks view |
| `v` | Switch to volumes view |

### Container Details
| Key | Action |
|-----|--------|
| `l` | Open logs |
| `s` | Open shell |
| `r` | Restart container |
| `S` | Stop or start container (context-sensitive) |
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `PgDn` | Scroll down 20 lines |
| `PgUp` | Scroll up 20 lines |
| `g` | Go to top |
| `G` | Go to bottom |
| `Esc` | Back to container list |

### Logs
| Key | Action |
|-----|--------|
| `Space` / `p` | Pause/resume auto-scroll |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `/` | Search within logs |
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `T` | Toggle timestamps on/off |
| `Ctrl+S` | Export logs to `/tmp/omdocker_logs_*.log` |
| `PgUp` / `PgDn` | Scroll 20 lines |

### Images
| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `r` / `Enter` | Run container from image (opens config form) |
| `d` | Remove image with confirmation |
| `D` | Remove all dangling (`<none>`) images |
| `p` | Prune all unused images |
| `/` | Activate fuzzy search |

### Image Run Form
| Key | Action |
|-----|--------|
| `Tab` / `↓` | Cycle through fields |
| `↑` | Previous field |
| Type | Fill in field value |
| `Backspace` | Delete last character |
| `a` | Toggle auto-remove (`--rm`) |
| `Enter` | Create and run container |
| `Esc` | Cancel and go back |

### Shell Config Form
| Key | Action |
|-----|--------|
| `Tab` / `↓` | Next field |
| `↑` | Previous field |
| Type | Fill in field value |
| `Backspace` | Delete last character |
| `Enter` | Save config + launch shell |
| `Esc` | Cancel and go back |

Three fields: **Shell** (`sh`, `bash`, `/bin/zsh`, etc.), **User** (empty=default, `host`, `root`, or `user:group`), **Workdir** (empty=default or custom path). Per-container config is persisted to `~/.config/omdocker/omdocker.toml`.

### Events
| Key | Action |
|-----|--------|
| `Space` | Pause/resume stream |
| `/` | Filter events |
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `PgUp` / `PgDn` | Scroll 20 lines |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `e` | Export events to `/tmp/omdocker_events_*.log` |

### Statistics
| Key | Action |
|-----|--------|
| `Esc` | Back to container list |
| `←` / `→` | Cycle sort column (name, CPU, memory, net RX, net TX, block read, block write, PIDs) |
| `t` | Toggle sort direction (ascending/descending) |

Live stats for running containers (CPU, memory, network, block I/O, PIDs), updated every 2s. Sorted columns show an arrow indicator.

### Networks
| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `d` | Delete selected network |
| `Esc` | Back to container list |

### Volumes
| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `d` | Delete selected volume |
| `Esc` | Back to container list |

## Project Structure

```
src/
  main.rs              — Async tokio event loop, shell exec, TUI lifecycle
  config.rs            — TOML config persistence (~/.config/omdocker/omdocker.toml)
  util.rs              — Shared helpers
  update.rs            — Self-update: GitHub release check, download, binary replace
  app/
    mode.rs            — Mode enum + ModeStack (back navigation, max depth 10)
    state.rs           — AppState with all sub-states
    event.rs           — AppEvent enum + data types + Command enum
    navigation.rs      — NavigationState (groups modal sub-states)
    reducer.rs         — Thin dispatcher, delegates to per-domain reducers
    reducers/          — Per-domain reducers (container, image, log, event, statistics, network, volume, shell, navigation)
    handlers/          — Per-domain input handlers (container, image, log, event, statistics, network, volume, shell, navigation)
  ui/
    mod.rs             — Shared render utilities (filter bar)
    containers.rs      — Container table with fuzzy search
    container_details.rs — Parsed metadata display with scrolling
    logs.rs            — Ring buffer (10k), follow/pause, search/highlight
    images.rs          — Image table + run config form (cmd/env/ports/volumes/name/rm)
    shell.rs           — Shell placeholder/waiting screen
    shell_config.rs    — Shell config form (shell, user, workdir)
    events.rs          — Live Docker event stream with type coloring + filter
    statistics.rs      — Live docker stats table (CPU/mem/net/block/PIDs)
    networks.rs        — Network list with delete
    volumes.rs         — Volume list with delete
    help.rs            — Full keybinding reference overlay
  docker/
    client.rs          — bollard Docker connection (local socket or env)
    containers.rs      — List, inspect, start, stop, restart, delete
    images.rs          — List, remove, create container (ContainerOpts)
    logs.rs            — Streaming log collector
    events.rs          — Docker event stream
    statistics.rs      — Docker stats collector
    networks.rs        — List, remove networks
    volumes.rs         — List, remove volumes
  search/
    fuzzy.rs           — fuzzy-matcher wrapper
  input/
    handler.rs         — Thin dispatcher, routes keys to app::handlers::* by mode
    keys.rs            — Action enum + global keybindings
  runtime/
    tasks.rs           — Async task spawners (pollers, actions, streams)
```

## Architecture

omdocker follows a unidirectional data flow:

```
Terminal input → AppEvent → Reducer → AppState → UI render
```

Async Docker operations (container polling, log streaming, event streaming) run on
background tokio tasks and communicate with the main loop via `mpsc` channels.

Each screen is an independent `Mode` in a state machine. Navigation between screens
uses a `ModeStack` (max depth 10) for back-button support. Destructive actions
(delete, remove) show a confirmation dialog before executing.

Shell access suspends the TUI entirely: the terminal exits raw mode/alternate screen,
`docker exec -it` runs as a blocking child process in the parent terminal, and on
exit the TUI re-initializes and resumes. Containers created with `--rm` are
automatically stopped on shell exit via `stop_on_exit`. Per-container shell
preferences (shell, user, workdir) are persisted to `~/.config/omdocker/omdocker.toml`.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI rendering (flexbox, scrollable blocks, styled text) |
| `crossterm` | Terminal backend + async event stream |
| `tokio` | Async runtime (pollers, streamers, channels) |
| `bollard` | Docker API client (containers, images, events, stats, networks, volumes) |
| `serde` / `serde_json` | Serialization for config and JSON metadata inspection |
| `toml` | Config persistence format (`~/.config/omdocker/omdocker.toml`) |
| `fuzzy-matcher` | Fuzzy search across container/image lists |
| `anyhow` | Error reporting and backtrace formatting |
| `chrono` | Timestamp formatting for logs and events |
| `futures-util` | Async stream utilities (log streaming, event streaming) |

## License

MIT
