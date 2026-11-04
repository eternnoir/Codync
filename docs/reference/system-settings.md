# System settings (`system.toml`)

The host reads `system.toml` from its data folder (`~/.codync/system.toml`; `CODYNC_HOME` or the Dev app's folder
when those apply) once at start-up. Every setting is optional and defaults to the value below, so a missing file
changes nothing. A file that can't be read or parsed, or that names an unknown setting, is logged and ignored as a
whole: the host starts with the defaults. Restart the host after editing it.

Code: `host/src/system_config.rs` (`SystemConfig`), passed around as `Hub::system`.

## `[compaction]`

When the host compacts a bot's main conversation or rolls it over into a new one
([design](../design/bot-memory.md#compaction), rules in `codync_mem::compaction::Config`).

| Setting | Default | Meaning |
| --- | --- | --- |
| `turn_end_cap` | `200000` | At the end of a turn, compact past three quarters of the window or this many tokens, whichever is less |
| `large` | `100000` | Idle compaction and the cold-cache rollover only happen past this much context |
| `before_expiry_minutes` | `5` | How long before a one-hour cache expires an idle conversation is compacted |
| `away_hours` | `8` | Away this long with `away_compactions` compactions behind it, the next message rolls over |
| `away_compactions` | `3` | See `away_hours` |
| `per_day` | `3` | Loop guard: at most this many host compactions a day |
| `thrash_percent` | `60` | Loop guard: no compaction while the last one left the context above this percentage of the trigger |
| `thrash_turns` | `5` | Loop guard: no compaction within this many turns of the last one |
| `quote_chars` | `2000` | The most of one message a handoff repeats word for word |

## `[team]`

Bot messages ([conversation runtime](../architecture/conversation-runtime.md)).

| Setting | Default | Meaning |
| --- | --- | --- |
| `max_request_hops` | `8` | Handoffs in one chain of bot requests before the user must start another |
| `max_pending_messages` | `64` | Bot messages waiting across all bots |
| `max_pending_messages_per_sender` | `16` | Bot messages one bot may have waiting |

## Example

```toml
[compaction]
turn_end_cap = 150000
per_day = 5

[team]
max_request_hops = 4
```
