# Architecture

Root summary — details in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/PLUGIN.md](docs/PLUGIN.md).

```
cmux workspaces ──titles──▶ cmux-moshi sync (sync.rs, mapping.rs, titles.rs) ──▶ tmux session names
                                                                               ▲
Moshi (Mosh/SSH client, MOSHI_CLIENT=1) ── login shell ── dashboard.rs (ratatui) ┘
LaunchAgent (launchagent.rs) runs periodic sync · doctor.rs diagnoses · cleanup.rs prunes
```
