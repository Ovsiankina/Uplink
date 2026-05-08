# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an **end-to-end automation testing framework** for the Uplink desktop application. It uses:
- **WebdriverIO** (v8.35.1) - test runner and orchestrator
- **Appium** (v2.5.3) - bridge between WebdriverIO and OS-level drivers
- **WinAppDriver** (Windows) / **Mac2 Driver** (macOS) - native app automation

Tests run against the compiled Uplink binary from `../Uplink` and validate UI behavior across Windows and macOS.

## Architecture

### High-Level Flow
```
Test Spec (tests/specs/*.spec.ts)
    ↓ uses
Screen Objects (tests/screenobjects/*.ts)
    ↓ locate elements via
UI Accessibility IDs (aria-labels set in Uplink source)
    ↓ controlled by
WebdriverIO + Appium   ← managed automatically by @wdio/appium-service
    ↓ controls
WinAppDriver (Windows) or Mac2Driver (macOS)
    ↓ automates
Uplink.exe / Uplink.app   ← launched automatically by WinAppDriver/Mac2Driver
```

**Important:** You do NOT manually start Appium or Uplink before running tests. WebdriverIO starts Appium via `@wdio/appium-service`, and Appium/WinAppDriver launches the Uplink binary automatically based on the `appium:app` capability.

### Key Directories

- **`config/`** - WebdriverIO configuration files
  - `wdio.shared.conf.ts` - shared settings (Appium service, timeouts, Mocha config)
  - `wdio.windows.app.conf.ts` - Windows local dev config
  - `wdio.windows.ci.conf.ts` - Windows CI/CD config
  - `wdio.mac.app.conf.ts` - macOS local dev config
  - `wdio.mac.multiremote.conf.ts` - macOS multi-instance (chat tests)

- **`tests/screenobjects/`** - Page Object Model (POM)
  - Defines UI locators and interaction methods per screen/component
  - Organized by feature: `chats/`, `settings/`, `account-creation/`, etc.
  - Each file maintains separate `SELECTORS_WINDOWS` and `SELECTORS_MACOS` objects
  - Selects the right set at runtime based on `process.env.DRIVER`

- **`tests/specs/`** - Actual test cases (Mocha describe/it blocks)
  - Numbered sequentially: `01-create-account.spec.ts`, `02-chats.spec.ts`, etc.
  - Each spec exports a function that is imported by a suite file

- **`tests/suites/`** - Ordered groupings of spec files
  - `01-UplinkTests.suite.ts` - macOS suite
  - `02-UplinkWindows.suite.ts` - Windows suite
  - These are the entry points that wdio actually runs

- **`tests/helpers/`** - Shared utilities
  - `commands.ts` - app lifecycle, user data, keyboard shortcuts
  - `constants.ts` - app paths, bundle IDs, driver names
  - `commandsClipboard.ts`, `commandsNewUser.ts`, `debugging.ts`

- **`tests/fixtures/users/`** - Pre-created user account caches for test setup

## Windows Setup (One-Time)

These are required once on a new Windows machine:

1. **Node.js v22** — v24 has compatibility issues with this project
2. **Rust** — to build Uplink (`cargo` must be in PATH)
3. **WinAppDriver** — installed at `C:\Program Files\Windows Application Driver\WinAppDriver.exe`
   - Download from: https://github.com/microsoft/WinAppDriver/releases
4. **Windows Developer Mode** — required by WinAppDriver to initialize
   - Settings → Privacy & Security → For developers → Developer Mode → ON
   - Without this, WinAppDriver exits with `0x80004005` and tests cannot run
5. **npm install** — installs Appium and all other dependencies locally (no global Appium needed)

## Running Tests

### Easiest Way: run-tests.ps1

```powershell
.\run-tests.ps1              # Run Windows tests
.\run-tests.ps1 -Rebuild     # Rebuild Uplink first, then run tests
.\run-tests.ps1 -Platform mac  # Run macOS tests
```

Or double-click `run-tests.bat` from File Explorer.

The script:
1. Checks Node.js and Uplink binary exist
2. Kills any leftover processes on ports 4723/4724
3. Runs `npm run windows.app` (WebdriverIO handles everything else)
4. Reports PASSED / FAILED

### Available npm Scripts

```bash
npm run windows.app      # Windows tests (local dev)
npm run windows.ci       # Windows tests (CI)
npm run mac.app          # macOS tests (local dev)
npm run mac.ci           # macOS tests (CI)
npm run mac.multiremote  # macOS multi-instance chat tests
npm run lint             # ESLint on config and tests
```

### Uplink Binary Location

The test config points to the debug build:
```
../Uplink/target/debug/uplink.exe   (Windows)
```
Defined in `tests/helpers/constants.ts` as `WINDOWS_APP`.
Build with `cargo build --package uplink` inside `../Uplink` if not present.

## Debugging

**Appium log** is written to `appium.log` in the project root during every run.
This is the first place to look when tests fail — it shows exactly what WinAppDriver/Appium did.

Common errors and their causes:

| Error | Cause | Fix |
|-------|-------|-----|
| `Developer mode is not enabled` | Windows Developer Mode is OFF | Enable in Settings |
| `WinAppDriver exited with code 2147500037` | Same as above | Same fix |
| `port may already be in use` | Old Appium process stuck on 4723 | Script kills it automatically; or kill manually |
| `power_shell has not been enabled` | Appium security config missing | Fixed in `wdio.shared.conf.ts` via `allowInsecure: ["power_shell"]` |
| `Cannot find uplink.exe` | Binary not built | Run `cargo build --package uplink` in `../Uplink` |

## Config Changes Made (vs Original Repo)

`config/wdio.shared.conf.ts`:
- Removed `command: "appium"` from the appium service — uses local npm install instead of global
- Added `allowInsecure: ["power_shell"]` — required for the prerun PowerShell cleanup command

`tests/helpers/constants.ts`:
- `WINDOWS_APP` changed from `C:\Program Files\uplink\uplink.exe` to the local dev build path

## Test Structure Patterns

### Screen Object

```typescript
import { WINDOWS_DRIVER } from "@helpers/constants";
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";

const SELECTORS_WINDOWS = { SIDEBAR: '[name="sidebar"]' };
const SELECTORS_MACOS   = { SIDEBAR: "~sidebar" };
const SELECTORS = process.env.DRIVER === WINDOWS_DRIVER ? SELECTORS_WINDOWS : SELECTORS_MACOS;

class ChatsSidebar extends UplinkMainScreen {
  get sidebar() { return $(SELECTORS.SIDEBAR); }
  async clickChat(name: string) { ... }
}
export default new ChatsSidebar();
```

### Spec File

```typescript
export default async function chatsTests() {
  it("sends a message", async () => {
    await ChatsSidebar.clickChat("TestUser");
    await InputBar.typeMessage("Hello");
    await InputBar.sendMessage();
    await expect(await MessageLocal.getLastMessage()).toHaveText("Hello");
  });
}
```

### Adding a New Test

1. Create `tests/specs/NN-feature.spec.ts` exporting an async function
2. Add or reuse screen objects in `tests/screenobjects/`
3. Import the function in the appropriate suite file (`02-UplinkWindows.suite.ts` for Windows)
4. Run `.\run-tests.ps1` to verify

### Running a Single Spec

Comment out the other imports in the suite file temporarily, or point the config's `specs` array at a single file.

## Locator Strategy

Preference order (most to least reliable):
1. **Accessibility IDs** — `~aria-label` (macOS) / `[name="aria-label"]` (Windows)
2. iOS class chain / Windows UI predicates
3. XPath (last resort — brittle, slow)

Aria-labels are defined in Uplink's Rust source (`kit/src`, `ui/src`). See `docs/ARIA_LABELS.md` for the full guide.

## CI/CD

Tests run on GitHub Actions on each PR:
- Builds Uplink for Windows and macOS
- Runs Windows single-instance, macOS single-instance, macOS multi-instance (chats)

See `.github/workflows/` for configuration.
Test reports land in `allure-results/` and `test-report/`.

## Resources

- **UI Locators Guide**: `docs/ARIA_LABELS.md`
- **Test Coverage Matrix**: `docs/TEST_COVERAGE.md`
- **WebdriverIO**: https://webdriver.io/
- **Appium**: https://appium.io/
