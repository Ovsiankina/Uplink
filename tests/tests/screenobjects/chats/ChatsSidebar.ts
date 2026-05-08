require("module-alias/register");
import { MACOS_DRIVER, WINDOWS_DRIVER } from "@helpers/constants";
import { rightClickOnMacOS, rightClickOnWindows } from "@helpers/commands";
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";
import { selectorContainer } from "@screenobjects/AppScreen";

let SELECTORS: selectorContainer = {};

const SELECTORS_COMMON: selectorContainer = {};

const SELECTORS_WINDOWS: selectorContainer = {
  CHAT_SEARCH_INPUT: '[name="chat-search-input"]',
  SIDEBAR: '[name="sidebar"]',
  SIDEBAR_CHATS_HEADER: "~chats-label",
  SIDEBAR_CHATS_HEADER_TEXT: "<Text>",
  SIDEBAR_CHATS_SECTION: "~chats",
  SIDEBAR_CHATS_USER: "<Group>",
  SIDEBAR_CHATS_USER_BADGE: '[name="User Badge"]',
  SIDEBAR_CHATS_USER_BADGE_NUMBER: '[name="badge-count"]',
  SIDEBAR_CHATS_USER_BADGE_NUMBER_VALUE: "<Text>",
  SIDEBAR_CHATS_USER_BADGE_TIME_AGO: '[name="badge-prefix"]',
  SIDEBAR_CHATS_USER_BADGE_TIME_AGO_VALUE: "<Text>",
  SIDEBAR_CHATS_USER_IMAGE: '[name="User Image"]',
  SIDEBAR_CHATS_USER_IMAGE_PROFILE: '[name="user-image-profile"]',
  SIDEBAR_CHATS_USER_IMAGE_WRAP: '[name="user-image-wrap"]',
  SIDEBAR_CHATS_USER_INFO: '[name="User Info"]',
  SIDEBAR_CHATS_USER_NAME: '[name="Username"]',
  SIDEBAR_CHATS_USER_NAME_VALUE: "<Text>",
  SIDEBAR_CHATS_USER_INDICATOR: '//Group[starts-with(@Name, "indicator")]',
  SIDEBAR_CHATS_USER_DO_NOT_DISTURB_INDICATOR:
    '[name="indicator-do-not-disturb"]',
  SIDEBAR_CHATS_USER_IDLE_INDICATOR: '[name="indicator-idle"]',
  SIDEBAR_CHATS_USER_OFFLINE_INDICATOR: '[name="indicator-offline"]',
  SIDEBAR_CHATS_USER_ONLINE_INDICATOR: '[name="indicator-online"]',
  SIDEBAR_CHATS_USER_PROFILE_TYPING: '[name="profile-typing"]',
  SIDEBAR_CHATS_USER_STATUS: '[name="User Status"]',
  SIDEBAR_CHATS_USER_STATUS_VALUE: "<Text>",
  SIDEBAR_CHILDREN: '[name="sidebar-children"]',
  SIDEBAR_CREATE_GROUP_CHAT_BUTTON: '[name="create-group-chat"]',
  SIDEBAR_GROUP_CHAT_IMAGE: '[name="user-image-group-wrap"]',
  SIDEBAR_GROUP_CHAT_PLUS_SOME: '[name="plus-some"]',
  SIDEBAR_SEARCH: '[name="sidebar-search"]',
  TOOLTIP: '[name="tooltip"]',
  TOOLTIP_TEXT: "<Text>",
};

const SELECTORS_MACOS: selectorContainer = {
  CHAT_SEARCH_INPUT: "~chat-search-input",
  SIDEBAR: "~sidebar",
  SIDEBAR_CHATS_HEADER: "~chats-label",
  SIDEBAR_CHATS_HEADER_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  SIDEBAR_CHATS_SECTION: "~Chats",
  SIDEBAR_CHATS_USER: "-ios class chain:**/XCUIElementTypeGroup",
  SIDEBAR_CHATS_USER_BADGE: "~User Badge",
  SIDEBAR_CHATS_USER_BADGE_NUMBER: "~badge-count",
  SIDEBAR_CHATS_USER_BADGE_NUMBER_VALUE:
    "-ios class chain:**/XCUIElementTypeStaticText",
  SIDEBAR_CHATS_USER_BADGE_TIME_AGO: "~badge-prefix",
  SIDEBAR_CHATS_USER_BADGE_TIME_AGO_VALUE:
    "-ios class chain:**/XCUIElementTypeStaticText",
  SIDEBAR_CHATS_USER_IMAGE: "~User Image",
  SIDEBAR_CHATS_USER_IMAGE_PROFILE: "~user-image-profile",
  SIDEBAR_CHATS_USER_IMAGE_WRAP: "~user-image-wrap",
  SIDEBAR_CHATS_USER_INFO: "~User Info",
  SIDEBAR_CHATS_USER_NAME: "~Username",
  SIDEBAR_CHATS_USER_NAME_VALUE:
    "-ios class chain:**/XCUIElementTypeStaticText",
  SIDEBAR_CHATS_USER_INDICATOR:
    '//XCUIElementTypeGroup[starts-with(@label, "indicator")]',
  SIDEBAR_CHATS_USER_DO_NOT_DISTURB_INDICATOR: "~indicator-do-not-disturb",
  SIDEBAR_CHATS_USER_IDLE_INDICATOR: "~indicator-idle",
  SIDEBAR_CHATS_USER_OFFLINE_INDICATOR: "~indicator-offline",
  SIDEBAR_CHATS_USER_ONLINE_INDICATOR: "~indicator-online",
  SIDEBAR_CHATS_USER_PROFILE_TYPING: "~profile-typing",
  SIDEBAR_CHATS_USER_STATUS: "~User Status",
  SIDEBAR_CHATS_USER_STATUS_VALUE:
    "-ios class chain:**/XCUIElementTypeStaticText",
  SIDEBAR_CHILDREN: "~sidebar-children",
  SIDEBAR_CREATE_GROUP_CHAT_BUTTON: "~create-group-chat",
  SIDEBAR_GROUP_CHAT_IMAGE: "~user-image-group-wrap",
  SIDEBAR_GROUP_CHAT_PLUS_SOME: "~plus-some",
  SIDEBAR_SEARCH: "~sidebar-search",
  TOOLTIP: "~tooltip",
  TOOLTIP_TEXT:
    "-ios class chain:**/XCUIElementTypeGroup/XCUIElementTypeStaticText",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

class ChatsSidebar extends UplinkMainScreen {
  constructor() {
    super(SELECTORS.SIDEBAR_CHATS_SECTION);
  }

  public get chatSearchInput() {
    return this.sidebarSearch.$(SELECTORS.CHAT_SEARCH_INPUT);
  }

  public get siderbarChatsHeader() {
    return this.sidebarChatsSection.$(SELECTORS.SIDERBAR_CHATS_HEADER);
  }

  public get siderbarChatsHeaderText() {
    return this.siderbarChatsHeader.$(SELECTORS.SIDEBAR_CHATS_HEADER_TEXT);
  }

  public get sidebar() {
    return $(SELECTORS.SIDEBAR);
  }

  public get sidebarChatsSection() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION);
  }

  public get sidebarChildren() {
    return $(SELECTORS.SIDEBAR_CHILDREN);
  }

  public get sidebarChatsUser() {
    return this.sidebarChatsSection.$(SELECTORS.SIDEBAR_CHATS_USER);
  }

  public get sidebarChatsUserBadge() {
    return this.sidebarChatsUser.$(SELECTORS.SIDEBAR_CHATS_USER_BADGE);
  }

  public get sidebarChatsUserBadgeNumber() {
    return this.sidebarChatsUserBadge.$(
      SELECTORS.SIDEBAR_CHATS_USER_BADGE_NUMBER,
    );
  }

  public get sidebarChatsUserBadgeNumberValue() {
    return this.sidebarChatsUserBadgeNumber.$(
      SELECTORS.SIDEBAR_CHATS_USER_BADGE_NUMBER_VALUE,
    );
  }

  public get sidebarChatsUserBadgeTimeAgo() {
    return this.sidebarChatsUserBadge.$(
      SELECTORS.SIDEBAR_CHATS_USER_BADGE_TIME_AGO,
    );
  }

  public get sidebarChatsUserBadgeTimeAgoValue() {
    return this.sidebarChatsUserBadgeTimeAgo.$(
      SELECTORS.SIDEBAR_CHATS_USER_BADGE_TIME_AGO_VALUE,
    );
  }

  public get sidebarChatsUserImage() {
    return this.sidebarChatsUser.$$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE);
  }

  public get sidebarChatsUserImageProfile() {
    return this.sidebarChatsSection.$(
      SELECTORS.SIDEBAR_CHATS_USER_IMAGE_PROFILE,
    );
  }

  public get sidebarChatsUserImageWrap() {
    return this.sidebarChatsUser.$$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP);
  }

  public get sidebarChatsUserInfo() {
    return this.sidebarChatsUser.$(SELECTORS.SIDEBAR_CHATS_USER_INFO);
  }

  public get sidebarChatsUserName() {
    return this.sidebarChatsUser.$(SELECTORS.SIDEBAR_CHATS_USER_NAME);
  }
  public get sidebarChatsUserNameValue() {
    return this.sidebarChatsUserName.$(SELECTORS.SIDEBAR_CHATS_USER_NAME_VALUE);
  }

  public get sidebarChatsUserIndicator() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_INDICATOR);
  }

  public get sidebarChatsUserDoNotDisturbIndicator() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_DO_NOT_DISTURB_INDICATOR);
  }

  public get sidebarChatsUserIdleIndicator() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IDLE_INDICATOR);
  }

  public get sidebarChatsUserOfflineIndicator() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_OFFLINE_INDICATOR);
  }

  public get sidebarChatsUserOnlineIndicator() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_ONLINE_INDICATOR);
  }

  public get sidebarChatsUserProfileTyping() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_PROFILE_TYPING);
  }

  public get sidebarChatsUserStatus() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS);
  }

  public get sidebarChatsUserStatusValue() {
    return $(SELECTORS.SIDEBAR_CHATS_SECTION)
      .$(SELECTORS.SIDEBAR_CHATS_USER)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS_VALUE);
  }

  public get sidebarCreateGroupChat() {
    return this.sidebarChatsSection.$(
      SELECTORS.SIDEBAR_CREATE_GROUP_CHAT_BUTTON,
    );
  }

  public get sidebarCreateGroupChatTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get sidebarCreateGroupChatTooltipText() {
    return this.sidebarCreateGroupChatTooltip.$(SELECTORS.TOOLTIP_TEXT);
  }

  public get sidebarGroupChatImage() {
    return $(SELECTORS.SIDEBAR_GROUP_CHAT_IMAGE);
  }

  public get sidebarGroupChatPlusSome() {
    return this.sidebarGroupChatImage.$(SELECTORS.SIDEBAR_GROUP_CHAT_PLUS_SOME);
  }

  public get sidebarGroupChatUserImages() {
    return this.sidebarGroupChatImage
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE);
  }

  public get sidebarGroupChatUserImageWraps() {
    return this.sidebarGroupChatImage.$$(
      SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP,
    );
  }

  public get sidebarGroupChatUserIndicator() {
    return this.sidebarGroupChatImage
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_INDICATOR);
  }

  public get sidebarGroupChatUserIndicatorOffline() {
    return this.sidebarGroupChatImage
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_OFFLINE_INDICATOR);
  }

  public get sidebarGroupChatsUserIndicatorOnline() {
    return this.sidebarGroupChatImage
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_ONLINE_INDICATOR);
  }

  public get sidebarSearch() {
    return $(SELECTORS.SIDEBAR_SEARCH);
  }

  // Validations or assertions

  async validateLastMessageDisplayed(message: string, username: string) {
    const sidebarUserLocator =
      await this.getExistingElementByAriaLabel(username);
    const sidebarStatusValue = await sidebarUserLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_STATUS)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS_VALUE);
    await sidebarStatusValue?.waitUntil(
      async () => {
        return (await sidebarStatusValue.getText()) === message;
      },
      {
        timeout: 15000,
        timeoutMsg:
          "Expected sidebar status value never include the message after 15 seconds",
      },
    );
  }

  async validateLastMessageTimeAgo(username: string) {
    const sidebarUserLocator =
      await this.getExistingElementByAriaLabel(username);
    const timeAgo = await sidebarUserLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_BADGE_TIME_AGO)
      .$(SELECTORS.SIDEBAR_CHATS_USER_BADGE_TIME_AGO_VALUE);
    await expect(timeAgo).toHaveText(
      expect.stringMatching(/(?:\d{1,2}\s+(?:second|minute)s?\s+ago|now)$/),
    );
  }

  async validateNoUnreadMessages(username: string) {
    const sidebarUserLocator =
      await this.getExistingElementByAriaLabel(username);
    await sidebarUserLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_BADGE)
      .waitForExist({
        reverse: true,
        timeout: 15000,
        timeoutMsg:
          "Expected badge number of unread messages is still displayed after 15 seconds",
      });
  }

  async validateNoSidebarChatsAreDisplayed() {
    await this.sidebarChatsUserImageProfile.waitForExist({
      reverse: true,
      timeout: 15000,
      timeoutMsg: "Sidebar chats are still displayed after 15 seconds",
    });
  }

  async validateSidebarChatIsNotDisplayed(username: string) {
    const locator: string =
      await this.getNonExistingElementByAriaLabel(username);
    await driver.waitUntil(
      async () => {
        return await this.sidebar.$(locator).waitForExist({ reverse: true });
      },
      {
        timeout: 15000,
        timeoutMsg: "Sidebar chats are still displayed after 15 seconds",
      },
    );
  }

  async validateNoSidebarGroupChatsAreDisplayed() {
    await driver.waitUntil(
      async () => {
        return await this.sidebarGroupChatImage.waitForExist({ reverse: true });
      },
      {
        timeout: 15000,
        timeoutMsg: "Sidebar group chats are still displayed after 15 seconds",
      },
    );
  }

  async validateNumberOfUnreadMessages(badgeNumber: string, username: string) {
    const sidebarUserLocator =
      await this.getExistingElementByAriaLabel(username);
    const badgeNumberDisplayed = await sidebarUserLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_BADGE_NUMBER)
      .$(SELECTORS.SIDEBAR_CHATS_USER_BADGE_NUMBER_VALUE);
    await expect(badgeNumberDisplayed).toHaveText(badgeNumber);
  }

  async validateUsernameIsDisplayed(username: string) {
    const sidebarUserLocator =
      await this.getExistingElementByAriaLabel(username);
    const usernameDisplayed = await sidebarUserLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_NAME)
      .$(SELECTORS.SIDEBAR_CHATS_USER_NAME_VALUE);
    await expect(usernameDisplayed).toHaveText(username);
  }

  async validateSidebarChatsIsShown() {
    const sidebarChats = await this.sidebarChatsUser;
    await sidebarChats.waitForExist();
  }

  // Waiting methods

  async waitForReceivingMessageOnSidebar() {
    (await this.sidebarChatsUserStatusValue).waitForExist({
      timeout: 30000,
      timeoutMsg: "Sidebar never displayed received messages after 30 seconds",
    });
  }

  // public get Sidebar Group elements

  async getExistingElementByAriaLabel(username: string) {
    const currentDriver = await this.getCurrentDriver();
    let locator: string;
    if (currentDriver === MACOS_DRIVER) {
      let element: WebdriverIO.Element;
      locator = "~" + username;
      element = await this.sidebar.$(locator);
      return element;
    } else {
      let element: WebdriverIO.Element;
      locator = '[name="' + username + '"]';
      element = await this.sidebar.$(locator);
      return element;
    }
  }

  async getNonExistingElementByAriaLabel(username: string) {
    const currentDriver = await this.getCurrentDriver();
    let locator: string;
    if (currentDriver === MACOS_DRIVER) {
      locator = "~" + username;
      return locator;
    } else if (currentDriver === WINDOWS_DRIVER) {
      locator = '[name="' + username + '"]';
      return locator;
    } else {
      locator = "";
      return locator;
    }
  }

  async waitForGroupToBeCreated(groupname: string) {
    const element: WebdriverIO.Element =
      await this.getExistingElementByAriaLabel(groupname);
    await element.waitForExist();
  }

  async waitForGroupToBeDeleted(groupname: string) {
    const element: string =
      await this.getNonExistingElementByAriaLabel(groupname);
    await $(SELECTORS.SIDEBAR).$(element).waitForExist({ reverse: true });
  }

  async getSidebarGroupPlusSome(groupname: string) {
    const groupLocator = await this.getExistingElementByAriaLabel(groupname);
    const plusSomeLocator = await groupLocator?.$(
      SELECTORS.SIDEBAR_GROUP_CHAT_PLUS_SOME,
    );
    await plusSomeLocator?.waitForExist();
    return plusSomeLocator;
  }

  async getSidebarGroupStatus(groupname: string) {
    const groupLocator = await this.getExistingElementByAriaLabel(groupname);
    const statusLocator = await groupLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_STATUS)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS_VALUE);
    await statusLocator?.waitForExist();
    return statusLocator;
  }

  // public get Sidebar User elements

  async getSidebarUserImage(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const imageLocator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE);
    await imageLocator?.waitForExist();
    return imageLocator;
  }

  async getSidebarUserStatus(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const statusLocator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_INFO)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS)
      .$(SELECTORS.SIDEBAR_CHATS_USER_STATUS_VALUE);
    await statusLocator?.waitForExist();
    return statusLocator;
  }

  async getSidebarUserIndicator(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const indicatorLocator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_INDICATOR);
    await indicatorLocator?.waitForExist();
    return indicatorLocator;
  }

  async getSidebarUserIndicatorDoNotDisturb(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const doNotDisturbLocator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_DO_NOT_DISTURB_INDICATOR);
    await doNotDisturbLocator?.waitForExist();
    return doNotDisturbLocator;
  }

  async getSidebarUserIndicatorIdle(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const idleLocator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IDLE_INDICATOR);
    await idleLocator?.waitForExist();
    return idleLocator;
  }

  async getSidebarUserIndicatorOffline(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const offlineLocator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_OFFLINE_INDICATOR);
    await offlineLocator?.waitForExist();
    return offlineLocator;
  }

  async getSidebarUserIndicatorOnline(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const onlineIndicator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_ONLINE_INDICATOR);
    await onlineIndicator?.waitForExist();
    return onlineIndicator;
  }

  async getSidebarUserIndicatorCurrentStatus(username: string) {
    const currentDriver = await this.getCurrentDriver();
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const userIndicator = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_INDICATOR);
    if (currentDriver === MACOS_DRIVER) {
      return await userIndicator?.getAttribute("label");
    } else if (currentDriver === WINDOWS_DRIVER) {
      return await userIndicator?.getAttribute("name");
    }
  }

  async getSidebarUserProfileTyping(username: string) {
    const userLocator = await this.getExistingElementByAriaLabel(username);
    const profileTyping = await userLocator
      ?.$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE_WRAP)
      .$(SELECTORS.SIDEBAR_CHATS_USER_IMAGE)
      .$(SELECTORS.SIDEBAR_CHATS_USER_PROFILE_TYPING);
    await profileTyping?.waitForExist();
    return profileTyping;
  }

  // Context menu methods

  async openContextMenuOnSidebar(username: string) {
    const imageToRightClick = await this.getSidebarUserIndicator(username);
    await this.hoverOnElement(imageToRightClick);
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === MACOS_DRIVER) {
      await rightClickOnMacOS(imageToRightClick);
    } else if (currentDriver === WINDOWS_DRIVER) {
      await rightClickOnWindows(imageToRightClick);
    }
  }

  async openContextOnFirstSidebarChat() {
    const imageToRightClick = await this.sidebarChatsUser;
    await this.hoverOnElement(imageToRightClick);
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === MACOS_DRIVER) {
      await rightClickOnMacOS(imageToRightClick);
    } else if (currentDriver === WINDOWS_DRIVER) {
      await rightClickOnWindows(imageToRightClick);
    }
  }

  async openContextMenuOnGroupChat(groupName: string) {
    const imageToRightClick =
      await this.getExistingElementByAriaLabel(groupName);
    await this.hoverOnElement(imageToRightClick);
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === MACOS_DRIVER) {
      await rightClickOnMacOS(imageToRightClick);
    } else if (currentDriver === WINDOWS_DRIVER) {
      await rightClickOnWindows(imageToRightClick);
    }
  }

  // Go to sidebar conversation
  async goToSidebarChat(username: string) {
    const imageToClick = await this.getSidebarUserImage(username);
    await imageToClick?.click();
  }

  async goToSidebarGroupChat(groupname: string) {
    const elementToClick = await this.getSidebarGroupStatus(groupname);
    await elementToClick?.click();
  }

  async goToSidebarFirstChat() {
    const imageToClick = await this.sidebarChatsUser;
    await imageToClick.click();
  }

  // Group Chats Methods

  async clickOnCreateGroupChat() {
    const createGroupButton = await this.sidebarCreateGroupChat;
    await createGroupButton.click();
  }

  async hoverOnCreateGroupButton() {
    const createGroupButton = await this.sidebarCreateGroupChat;
    await this.hoverOnElement(createGroupButton);
  }

  // Search bar methods

  async clearSidebarSearchInput() {
    const chatSearchInput = await this.chatSearchInput;
    await chatSearchInput.clearValue();
  }

  async typeOnSidebarSearchInput(text: string) {
    const chatSearchInput = await this.chatSearchInput;
    await chatSearchInput.clearValue();
    await chatSearchInput.setValue(text);
    const chatSearchInputValue = await chatSearchInput.getText();
    if (chatSearchInputValue !== text) {
      await this.typeOnSidebarSearchInput(text);
    }
  }
}

export default new ChatsSidebar();
