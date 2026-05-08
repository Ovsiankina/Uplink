# Manual E2E Test — Chat Replies

**Module tested:** `ui/src/layouts/chats/presentation/messages/mod.rs`  
**Scenario:** Two users exchange messages and replies in a real-time peer-to-peer chat.

---

## Prerequisites

- Uplink desktop app installed and runnable on the test machine.
- Two user accounts already created: **ChatUserA** and **ChatUserB** (they must be mutual friends).
- Two separate Uplink instances can run simultaneously (e.g. two different profile paths, or two machines).
- Both instances are closed before starting.

---

## Setup

1. Launch the **first** Uplink instance (ChatUserA).
2. Enter the PIN when prompted and wait for the main screen to load.
3. Launch the **second** Uplink instance (ChatUserB).
4. Enter the PIN when prompted and wait for the main screen to load.
5. On the **second instance** (ChatUserB), open the sidebar and click on the existing conversation with **ChatUserA**.
   - The chat input bar must be visible at the bottom of the screen.

---

## Test Steps

### 1 — ChatUserB sends a text message

1. On the **second instance** (ChatUserB), click inside the message input bar.
2. Type: `Testing...`
3. Press **Enter** or click the **Send** button.
4. **Expected:** The message `Testing...` appears immediately in the chat area on ChatUserB's side, displayed as a sent (local) message bubble.

---

### 2 — ChatUserA receives the message

1. Switch to the **first instance** (ChatUserA).
2. Open the sidebar and click on the conversation with **ChatUserB** if it is not already active.
3. **Expected:** The message `Testing...` is visible in the chat area, displayed as a received (remote) message bubble.

---

### 3 — ChatUserA opens the reply popup

1. On the **first instance** (ChatUserA), right-click (or long-press) on the received message `Testing...`.
2. In the context menu that appears, click **Reply**.
3. **Expected:** A reply prompt banner appears above the input bar, showing the original message text `Testing...` inside it.

---

### 4 — ChatUserA closes the reply popup without sending

1. Click the **Close (×)** button on the reply prompt banner.
2. **Expected:** The reply banner disappears. The input bar returns to its normal empty state.

---

### 5 — ChatUserA sends an actual reply

1. On the **first instance** (ChatUserA), right-click the received message `Testing...` again.
2. Click **Reply** in the context menu.
3. **Expected:** The reply prompt banner reappears, quoting `Testing...`.
4. In the input bar, type: `Reply`
5. Press **Enter** or click the **Send** button.
6. **Expected:**
   - The reply prompt banner disappears after sending.
   - A new message group appears in the chat with two parts:
     - A small quoted block above showing `Testing...` (the message being replied to).
     - The main message bubble below showing `Reply`.

---

### 6 — ChatUserA verifies the reply layout

1. Look at the message group that contains the reply on **ChatUserA**'s screen.
2. **Expected — quoted block:** The text `Testing...` is visible in a smaller, quoted style above the reply.
3. **Expected — reply bubble:** The text `Reply` is visible in the main message bubble.
4. **Expected — timestamp:** The message group header shows a timestamp in the format `ChatUserA - X seconds ago` or `ChatUserA - now`.
5. **Expected — avatar:** ChatUserA's profile picture is displayed next to the message group.

---

### 7 — ChatUserB receives the reply

1. Switch to the **second instance** (ChatUserB).
2. **Expected:**
   - A new message group appears showing a quoted block with `Testing...` above a bubble with `Reply`.
   - The text in the quoted block reads exactly `Testing...`.
   - The text in the main bubble reads exactly `Reply`.

---

### 8 — ChatUserB verifies timestamp and avatar on the received reply

1. On the **second instance** (ChatUserB), look at the message group header for the reply.
2. **Expected — timestamp:** Displays `ChatUserA - X seconds ago` or `ChatUserA - now`.
3. **Expected — avatar:** ChatUserA's profile picture is displayed next to the message group.

---

### 9 — ChatUserB replies to their own earlier message (self-reply)

1. On the **second instance** (ChatUserB), right-click on the sent message `Testing...` (the one ChatUserB originally sent).
2. Click **Reply** in the context menu.
3. **Expected:** The reply prompt banner appears quoting `Testing...`.
4. In the input bar, type: `SelfReply`
5. Press **Enter** or click the **Send** button.
6. **Expected:**
   - The reply banner disappears.
   - A new message group appears with:
     - A small quoted block showing `Testing...`.
     - A main message bubble showing `SelfReply`.

---

## Pass Criteria

| # | Check | Result |
|---|-------|--------|
| 1 | `Testing...` appears as sent bubble on ChatUserB | ☐ |
| 2 | `Testing...` appears as received bubble on ChatUserA | ☐ |
| 3 | Reply prompt opens showing `Testing...` | ☐ |
| 4 | Reply prompt closes without sending | ☐ |
| 5 | Reply `Reply` is sent with quoted block above | ☐ |
| 6 | Timestamp shows `ChatUserA - X seconds/now` on local side | ☐ |
| 7 | Avatar visible next to message group on local side | ☐ |
| 8 | ChatUserB sees quoted block `Testing...` + bubble `Reply` | ☐ |
| 9 | Timestamp shows `ChatUserA - X seconds/now` on remote side | ☐ |
| 10 | Avatar visible next to message group on remote side | ☐ |
| 11 | Self-reply `SelfReply` appears with quoted `Testing...` above it | ☐ |

All 11 checks must pass for this test to be considered **PASSED**.

---

## Teardown

- Close both Uplink instances.
