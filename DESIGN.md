# Design

Login-shell **dashboard** (ratatui TUI) when connecting from Moshi, plus an optional workspace picker in the cmux sidebar.
- **Native first**: inherit the cmux sidebar chrome and the Ghostty/terminal theme (colours, fonts, spacing); no custom palette.
- Status is shown as compact pills; colour carries state only together with a text/glyph (never colour alone).
- Must stay legible in both light and dark cmux themes; test at narrow sidebar widths.
- Friendly workspace titles are the primary label; raw tmux session ids are secondary.
