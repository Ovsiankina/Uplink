# Features to Test

| | Feature | Test type | Target |
|---|---------|-----------|--------|
| A | Message text formatting | Unit tests (50+) | `kit/src/components/message/mod.rs` |
| Y | Profile update flow | Integration tests (5+) | `ui/src/components/settings/sub_pages/profile/mod.rs` + mocked backend |

---

## A — Message text formatting

**What it is:** The pipeline that transforms a raw message string into displayable HTML. Covers ASCII-emoji conversion, markdown rendering, HTML escaping, link detection, and multi-emoji detection.

**Completeness: 92 / 100** — fully functional, minor edge cases around mention replacement.

### Files

| File | What to test |
|------|-------------|
| `kit/src/components/message/mod.rs` | `format_text()`, `replace_emojis()`, `markdown()`, `is_only_emojis()`, `process_string()`, `wrap_links_with_a_tags()`, `stack_processor()` |
| `common/src/state/utils.rs` | `parse_mentions()`, `mention_replacement_pattern()` — needed for `format_text()` with mention data (requires a fake `State` stub) |

---

## Y — Profile update flow

**What it is:** The chain from user input validation through command dispatch to state update. A user edits their username, status message, or online status — the input is validated, a `MultiPassCmd` is sent through a channel, and the state is updated on success.

**Completeness: 92 / 100** — fully functional, two minor TODOs around image empty detection and cache flag persistence.

### Files

| File | What to test |
|------|-------------|
| `kit/src/elements/input/mod.rs` | `Validation` struct — the validation rules applied to every profile field |
| `ui/src/layouts/log_in/enter_username.rs` | `MIN_USERNAME_LEN = 4`, `MAX_USERNAME_LEN = 32` — authoritative constants |
| `ui/src/components/settings/sub_pages/profile/mod.rs` | `ChanCmd` dispatch logic, error handling (success toast vs. error toast), `transform_file_into_base64_image()` |
| `common/src/warp_runner/manager/commands/multipass_commands.rs` | `MultiPassCmd` variants used by the profile page — **mock this channel** as the integration boundary |
| `common/src/state/identity.rs` | State update after a successful command — verify the in-memory identity reflects the change |
