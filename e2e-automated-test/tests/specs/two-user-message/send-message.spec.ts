require("module-alias/register");
import { createNewUser } from "@helpers/commandsNewUser";
import CreatePinScreen from "@screenobjects/account-creation/CreatePinScreen";
import ChatsSidebar from "@screenobjects/chats/ChatsSidebar";
import FriendsScreen from "@screenobjects/friends/FriendsScreen";
import InputBar from "@screenobjects/chats/InputBar";
import WelcomeScreen from "@screenobjects/welcome-screen/WelcomeScreen";

// E2E test for the ":)" → "🙂" emoji conversion in
// `kit/src/components/message/mod.rs` (replace_emojis / format_text).
//
// Uplink runs with --with-mock so we get 20 fake friends + chats locally.
// Pass condition is intentionally lenient: it just verifies a local message
// bubble appears after we send. The visual emoji conversion is what's being
// demonstrated on screen — the assertion only proves the spec ran end-to-end.
export default function emojiSendTest() {
  it("Setup: complete PIN + account-creation flow", async () => {
    await CreatePinScreen.waitForIsShown(true);
    await createNewUser("MockTester");
    await WelcomeScreen.waitForIsShown(true);
  });

  it("Open the first mock-friend chat from the friends list", async () => {
    await WelcomeScreen.goToFriends();
    await FriendsScreen.validateFriendsScreenIsShown();

    const chatBtn = await FriendsScreen.chatWithFriendButton;
    await chatBtn.waitForExist({ timeout: 30000 });
    await chatBtn.click();

    // Wait for the chat panel to be ready for input.
    await InputBar.waitForIsShown(true);
    // Mock chats can have up to 20 pre-existing random messages, so give the
    // thread time to render before we send.
    await browser.pause(3000);
  });

  it("Send ':)' — Uplink renders it as 🙂 in the sidebar last-message preview", async () => {
    await InputBar.clickOnInputBar();
    await InputBar.typeMessageOnInput(":)");
    await InputBar.clickOnSendMessage();

    // The main chat thread is stuck on "Fetching more messages..." in mock
    // mode, so the message bubble never renders. But the sidebar's
    // last-message preview DOES update with the converted emoji — that's
    // what we assert on. The first sidebar chat is the one we just opened
    // and sent into, so its status text should contain "🙂" once the send
    // dispatches.
    await browser.waitUntil(
      async () => {
        try {
          const statusEl = await ChatsSidebar.sidebarChatsUserStatusValue;
          const text = await statusEl.getText();
          return text.includes("🙂");
        } catch (e) {
          return false;
        }
      },
      {
        timeout: 20000,
        interval: 500,
        timeoutMsg:
          "Sidebar last-message preview never showed the 🙂 conversion after send",
      },
    );
  });
}
