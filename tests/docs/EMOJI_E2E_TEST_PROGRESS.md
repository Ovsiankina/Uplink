# Emoji E2E Test — Progress Log

## Goal

Build one end-to-end automated test on Windows that exercises the `:)` →
`🙂` conversion logic in `Uplink/kit/src/components/message/mod.rs`
(`replace_emojis` / `format_text`). Test goal is a school-project video
demo — brittle code is acceptable, the test just needs to run successfully
once.

## Current state (end of day 2026-05-06)

The test runs through Uplink's full UI flow end-to-end:
1. Uplink launches (with `--with-mock --discovery disable`)
2. PIN screen → create-account flow completes successfully
3. App lands on the main UI populated by mock state (20 fake friends + chats)
4. Test navigates to Friends, clicks the first friend's "Chat With Friend"
   button, lands in the chat with the input bar visible
5. Test types `:)` and clicks send

**The only thing broken is the final assertion.** The test currently
checks for an element with `[name="message-local"]` after sending — that
selector doesn't match anything within 30 s, so the test fails. The
emoji conversion itself is happening (visible on screen); we just don't
have the right locator to confirm it programmatically yet.

## Confirmed working

- `--with-mock` as a CLI flag (clap converts the Rust field `with_mock` to
  kebab-case for the actual flag — `--with_mock` with underscore was
  silently rejected and caused Uplink to exit immediately)
- `--discovery disable` to bypass the dead public Shuttle peer
- Single-instance Windows automation through PIN → account creation →
  navigate to Friends → click chat with friend → input bar ready

## What was fixed along the way

Several real bugs surfaced and were fixed today:

### 1. `launchAppWindows` double-prefixed the absolute exe path

`tests/helpers/commands.ts` was calling
`join(process.cwd(), appLocation)` where `appLocation` was already an
absolute path (`WINDOWS_APP` from `tests/helpers/constants.ts`). On
Windows, `path.join` happily produces malformed paths like
`C:\testing-uplink\C:\Uplink\target\debug\uplink.exe`. WinAppDriver
silently fell back to launching from the session cap without `--path`,
so any "second instance" actually used the default `~/.uplink` data
dir and showed the first user's account. **Fix:** dropped the `join` —
`appLocation` is already absolute.

### 2. Uplink hangs on init without a reachable peer

Uplink's default `Discovery::Shuttle` mode targets a hard-coded public
peer (`/ip4/159.65.41.31/tcp/8848`). When that's unreachable the UI
thread blocks during cold-start and PIN entry / create-account never
becomes responsive. **Fix:** pass `--discovery disable` for the local
test runs.

### 3. `windows: launchApp` drops `appArguments`

Confirmed from `appium.log` line 1684: when `windows: launchApp` proxies
to WinAppDriver it sends `body: {}`. The extension only relaunches the
exe with the *session*'s original capabilities — `--path .uplinkUserB`
sent at runtime is never honored. This is why the original `launchApp`
based two-instance flow couldn't actually open a second account. We
worked around it briefly with `child_process.spawn`, but that hit the
next problem.

### 4. WinAppDriver attaches to one process per session

Each WebDriver session is bound to the process tree of the app it was
created against. `getWindowHandles()` only returns windows from that
tree. A separately-spawned `uplink.exe` is invisible to the existing
session — no element queries, no automation. This is the fundamental
reason "two-user messaging on Windows" needs either WebdriverIO
multiremote (two parallel sessions) or sequential `reloadSession`. The
existing macOS multi-instance tests sidestep this because Mac2Driver
can target apps by `bundleId`.

### 5. The Windows CI suite never ran multi-user specs

`tests/suites/MainTests/02-UplinkWindows.suite.ts` (the suite Windows CI
runs) only includes single-user specs. The multi-user chat specs
(`01-create-accounts-and-friends.spec`, `04-message-input.spec`, etc.)
were always macOS-only. There was no "original Windows multi-user test"
to revive.

### 6. Pivot to `--with-mock` (single instance)

`generate_mock()` in `Uplink/common/src/testing/mock.rs` produces 20
random friends and chats locally, with no warp/network involvement.
Sending a message in mock mode short-circuits to
`Action::MockSend(active_chat_id, msg)` — local state mutation only.
That gives us everything we need (a chat to send into, no shuttle
required) using a single Uplink instance, which the existing
WinAppDriver setup handles fine.

## Files changed

- `tests/helpers/commands.ts` — fixed double-prefix path bug; reverted
  the `child_process.spawn` workaround now that we're single-instance;
  kept the per-app window-handle tracking (harmless when only one
  window exists)
- `config/wdio.windows.chats.conf.ts` — single-instance setup with
  `--with-mock --discovery disable` capability args, `kill-port` +
  `taskkill /IM uplink.exe /F` cleanup in `onComplete`, cleans
  `~/.uplink` and `~/.uplinkUserB` in `onPrepare`
- `tests/suites/Chats/02-WindowsMessages.suite.ts` — points at the
  single new spec
- `tests/specs/two-user-message/send-message.spec.ts` — the actual
  spec (single-user, mock-mode, sends `:)` and checks for a local
  message bubble)
- `msg-test.ps1` — runner script (kills stale ports, invokes wdio)

## TODOs for tomorrow

### TODO 1 — Fix the assertion

The full automation pipeline works end-to-end up to and including
`InputBar.clickOnSendMessage()`. The only failing step is the final
"verify the message appeared" check.

- Current attempt: `await $('[name="message-local"]').waitForExist(...)`
  — times out, so either that aria-label doesn't exist or it has a
  different name on Windows.
- Action: inspect the rendered DOM after sending (run Uplink with
  `--with-mock` manually, send a message, use Appium Inspector or a
  one-shot debug script that dumps `await $('//*').getAttribute('name')`)
  to find the actual aria-label of the local message bubble or its
  inner text element.
- The `:)` → `🙂` conversion produces
  `<span class="big-emoji">🙂</span>` (because `is_only_emojis()` is
  true after replacement), which won't match the standard
  `<Text>` selector used by `MessageLocal.getCustomMessageContents`.
  The fix is probably to query the big-emoji span directly, or just
  check that the input bar got cleared (`InputBar.inputCharCounterText`
  becomes `"0"` after a successful send — that's the cleanest pass
  signal that doesn't depend on message rendering at all).

### TODO 2 — Strip the project to the minimum

Right now the project still contains all the macOS-specific configs,
specs, screen objects, fixtures, and helpers that aren't used by our
single test. Goal: delete everything that isn't needed to run
`msg-test.ps1` end-to-end on Windows.

Keep:
- `msg-test.ps1`
- `package.json`, `package-lock.json`, `tsconfig.json`,
  `node_modules/` (managed)
- `config/wdio.shared.conf.ts`
- `config/wdio.windows.chats.conf.ts`
- `tests/suites/Chats/02-WindowsMessages.suite.ts`
- `tests/specs/two-user-message/send-message.spec.ts`
- `tests/helpers/` — only the functions actually called by the spec
  (`createNewUser` and its transitive deps)
- `tests/screenobjects/` — only the screens actually used
  (CreatePinScreen, CreateOrImportScreen, CreateUserScreen,
  SaveRecoverySeedScreen, WelcomeScreen, FriendsScreen, InputBar,
  MessageLocal, AppScreen, UplinkMainScreen)
- `tests/fixtures/users/` — likely not needed at all in mock mode;
  verify and delete if unused
- `docs/` — keep this file; consider trimming the others

Delete:
- All other configs (`wdio.mac.*`, `wdio.windows.app.conf.ts`,
  `wdio.windows.ci.conf.ts`, `wdio.windows.onetime.conf.ts`)
- All other suites and specs
- All unused screen objects
- `run-tests.bat` and any other macOS-targeted runners
- macOS-specific helpers and constants
- `.github/workflows/` (we're not running CI)
- `patches/`, `appium.log`, `allure-results/`, `test-report/`,
  `test-results/`, `test-screenshots/` — generated/dev artifacts

Resulting tree should be small enough that the whole project is one
glance.
