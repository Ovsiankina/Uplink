require("module-alias/register");
import AppScreen from "@screenobjects/AppScreen";
import {
  hoverOnMacOS,
  hoverOnWindows,
  leftClickOnMacOS,
  leftClickOnWindows,
} from "@helpers/commands";
import { MACOS_DRIVER, WINDOWS_DRIVER } from "@helpers/constants";
import { selectorContainer } from "@screenobjects/AppScreen";

const SELECTORS_COMMON: selectorContainer = {
  RELEASE_INDICATOR: "~alpha",
};

const SELECTORS_WINDOWS: selectorContainer = {
  BACK_BUTTON: '[name="back-button"]',
  BUTTON_BADGE: '[name="Button Badge"]',
  BUTTON_BADGE_TEXT: "<Text>",
  BUTTON_NAV: '[name="button-nav"]',
  BUTTON_NAV_TOOLTIP: '[name="tooltip"]',
  BUTTON_NAV_TOOLTIP_TEXT: "<Text>",
  CHATS_BUTTON: '[name="chats-button"]',
  CONTEXT_MENU: '[name="Context Menu"]',
  CONTEXT_MENU_OPEN_WEB_INSPECTOR: '[name="open-devtools-context"]',
  FILES_BUTTON: '[name="files-button"]',
  FRIENDS_BUTTON: '[name="friends-button"]',
  HAMBURGER_BUTTON: '[name="hamburger-button"]',
  LOADING_SPINNER: '[name="loader"]',
  MODAL: '[name="modal"]',
  RELEASE_INDICATOR_TEXT: "<Text>",
  SETTINGS_BUTTON: '[name="settings-button"]',
  TOAST_NOTIFICATION: '[name="Toast Notification"]',
  TOAST_NOTIFICATION_CLOSE: '[name="close-toast"]',
  TOAST_NOTIFICATION_TEXT: '[name="toast-content"]',
  TOOLTIP: '[name="tooltip"]',
  TOOLTIP_TEXT: "//Group/Text",
  UPDATE_AVAILABLE: '[name="update-available"]',
  UPDATE_AVAILABLE_TEXT: "<Text>",
  UPDATE_MENU_DISMISS: '[name="update-menu-dismiss"]',
  UPDATE_MENU_DOWNLOAD: '[name="update-menu-download"]',
  WINDOW: "~main",
};

const SELECTORS_MACOS: selectorContainer = {
  BACK_BUTTON: "~back-button",
  BUTTON_BADGE: "~Button Badge",
  BUTTON_BADGE_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  BUTTON_NAV: "~button-nav",
  BUTTON_NAV_TOOLTIP: "~tooltip",
  BUTTON_NAV_TOOLTIP_TEXT:
    "-ios class chain:**/XCUIElementTypeGroup/XCUIElementTypeStaticText",
  CHATS_BUTTON: "~chats-button",
  CONTEXT_MENU: "~Context Menu",
  CONTEXT_MENU_OPEN_WEB_INSPECTOR: "~open-devtools-context",
  FILES_BUTTON: "~files-button",
  FRIENDS_BUTTON: "~friends-button",
  HAMBURGER_BUTTON: "~hamburger-button",
  LOADING_SPINNER: "~loader",
  MODAL: "~modal",
  RELEASE_INDICATOR_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  SETTINGS_BUTTON: "~settings-button",
  TOAST_NOTIFICATION: "~Toast Notification",
  TOAST_NOTIFICATION_CLOSE: "~close-toast",
  TOAST_NOTIFICATION_TEXT: "~toast-content",
  TOOLTIP: "~tooltip",
  TOOLTIP_TEXT:
    "-ios class chain:**/XCUIElementTypeGroup/XCUIElementTypeStaticText",
  UPDATE_AVAILABLE: "~update-available",
  UPDATE_AVAILABLE_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  UPDATE_MENU_DISMISS: "~update-menu-dismiss",
  UPDATE_MENU_DOWNLOAD: "~update-menu-download",
  WINDOW: "-ios class chain:**/XCUIElementTypeWindow",
};

let SELECTORS: selectorContainer = {
  BACK_BUTTON: "",
  BUTTON_BADGE: "",
  BUTTON_BADGE_TEXT: "",
  BUTTON_NAV: "",
  BUTTON_NAV_TOOLTIP: "",
  BUTTON_NAV_TOOLTIP_TEXT: "",
  CHATS_BUTTON: "",
  CONTEXT_MENU: "",
  CONTEXT_MENU_OPEN_WEB_INSPECTOR: "",
  FILES_BUTTON: "",
  FRIENDS_BUTTON: "",
  HAMBURGER_BUTTON: "",
  LOADING_SPINNER: "",
  MODAL: "",
  RELEASE_INDICATOR: "",
  RELEASE_INDICATOR_TEXT: "",
  SETTINGS_BUTTON: "",
  TOAST_NOTIFICATION: "",
  TOAST_NOTIFICATION_CLOSE: "",
  TOAST_NOTIFICATION_TEXT: "",
  TOOLTIP: "",
  TOOLTIP_TEXT: "",
  UPDATE_AVAILABLE: "",
  UPDATE_AVAILABLE_TEXT: "",
  UPDATE_MENU_DISMISS: "",
  UPDATE_MENU_DOWNLOAD: "",
  WINDOW: "",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

export default class UplinkMainScreen extends AppScreen {
  constructor(locator: string = SELECTORS.WINDOW) {
    super(locator);
  }

  public get backButton() {
    return $(SELECTORS.BACK_BUTTON);
  }

  public get buttonNav() {
    return $(SELECTORS.BUTTON_NAV);
  }

  public get buttonNavBarButtonBadge() {
    return this.buttonNav.$(SELECTORS.BUTTON_BADGE);
  }

  public get buttonNavBarButtonBadgeText() {
    return this.buttonNavBarButtonBadge.$(SELECTORS.BUTTON_BADGE_TEXT);
  }

  public get chatsButton() {
    return this.buttonNav.$(SELECTORS.CHATS_BUTTON);
  }

  public get chatsButtonTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get chatsButtonTooltipText() {
    return $(SELECTORS.TOOLTIP).$(SELECTORS.BUTTON_NAV_TOOLTIP_TEXT);
  }

  public get contextMenu() {
    return $(SELECTORS.CONTEXT_MENU);
  }

  public get contextMenuOpenWebInspector() {
    return this.contextMenu.$(SELECTORS.CONTEXT_MENU_OPEN_WEB_INSPECTOR);
  }

  public get filesButton() {
    return this.buttonNav.$(SELECTORS.FILES_BUTTON);
  }

  public get filesButtonTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get filesButtonTooltipText() {
    return $(SELECTORS.TOOLTIP).$(SELECTORS.BUTTON_NAV_TOOLTIP_TEXT);
  }

  public get friendsButton() {
    return this.buttonNav.$(SELECTORS.FRIENDS_BUTTON);
  }

  public get friendsButtonTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get friendsButtonTooltipText() {
    return $(SELECTORS.TOOLTIP).$(SELECTORS.BUTTON_NAV_TOOLTIP_TEXT);
  }

  public get hamburgerButton() {
    return $(SELECTORS.HAMBURGER_BUTTON);
  }

  public get loaderSpinner() {
    return $(SELECTORS.LOADING_SPINNER);
  }

  public get modal() {
    return $(SELECTORS.MODAL);
  }

  public get releaseIndicator() {
    return $(SELECTORS.RELEASE_INDICATOR);
  }

  public get releaseIndicatorText() {
    return this.releaseIndicator.$(SELECTORS.RELEASE_INDICATOR_TEXT);
  }

  public get settingsButton() {
    return this.buttonNav.$(SELECTORS.SETTINGS_BUTTON);
  }

  public get settingsButtonTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get settingsButtonTooltipText() {
    return $(SELECTORS.TOOLTIP).$(SELECTORS.BUTTON_NAV_TOOLTIP_TEXT);
  }

  public get toastNotifications() {
    return $$(SELECTORS.TOAST_NOTIFICATION);
  }

  public get toastNotification() {
    return $(SELECTORS.TOAST_NOTIFICATION);
  }

  public get toastNotificationClose() {
    return this.toastNotification.$(SELECTORS.TOAST_NOTIFICATION_CLOSE);
  }

  public get toastNotificationText() {
    return this.toastNotification.$(SELECTORS.TOAST_NOTIFICATION_TEXT);
  }

  public get updateAvailable() {
    return $(SELECTORS.UPDATE_AVAILABLE);
  }

  public get updateAvailableText() {
    return this.updateAvailable.$(SELECTORS.UPDATE_AVAILABLE_TEXT);
  }

  public get updateMenuDismiss() {
    return $(SELECTORS.UPDATE_MENU_DISMISS);
  }

  public get updateMenuDownload() {
    return $(SELECTORS.UPDATE_MENU_DOWNLOAD);
  }

  public get window() {
    return $(SELECTORS.WINDOW);
  }

  // Toast Notifications methods

  async closeToastNotification() {
    const toast = await this.toastNotificationClose;
    await toast.click();
  }

  async getToastNotificationText() {
    const toastText = await this.toastNotificationText;
    const toastTextValue = await toastText.getText();
    return toastTextValue;
  }

  async waitUntilNotificationIsClosed() {
    await this.toastNotification.waitForExist({
      reverse: true,
    });
  }

  // Clicking on common elements methods

  async clickOnBackButton() {
    const button = await this.backButton;
    await button.click();
  }

  async clickOnHamburgerButton() {
    const button = await this.hamburgerButton;
    await button.click();
  }

  async clickOnReleaseIndicator() {
    const release = await this.releaseIndicator;
    await release.click();
  }

  async clickOnUpdateAvailable() {
    const updateAvailable = await this.updateAvailable;
    await updateAvailable.click();
  }

  // NavBar methods

  async goToFiles() {
    const button = await this.filesButton;
    await button.click();
  }

  async goToFriends() {
    const button = await this.friendsButton;
    await button.click();
  }

  async goToMainScreen() {
    const chatsButton = await this.chatsButton;
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === MACOS_DRIVER) {
      await leftClickOnMacOS(chatsButton);
    } else if (currentDriver === WINDOWS_DRIVER) {
      await leftClickOnWindows(chatsButton);
    }
  }

  async goToSettings() {
    const currentDriver = await this.getCurrentDriver();
    const button = await this.settingsButton;
    await this.hoverOnElement(button);
    if (currentDriver === MACOS_DRIVER) {
      await leftClickOnMacOS(button);
    } else if (currentDriver === WINDOWS_DRIVER) {
      await leftClickOnWindows(button);
    }
  }

  // Hovering methods

  async hoverOnElement(element: WebdriverIO.Element | undefined) {
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === MACOS_DRIVER) {
      await hoverOnMacOS(element);
    } else if (currentDriver === WINDOWS_DRIVER) {
      await hoverOnWindows(element);
    }
  }

  async hoverOnChatsButton() {
    const chatsButton = await this.chatsButton;
    await this.hoverOnElement(chatsButton);
  }

  async hoverOnFilesButton() {
    const filesButton = await this.filesButton;
    await this.hoverOnElement(filesButton);
  }

  async hoverOnFriendsButton() {
    const friendsButton = await this.friendsButton;
    await this.hoverOnElement(friendsButton);
  }

  async hoverOnSettingsButton() {
    const settingsButton = await this.settingsButton;
    await this.hoverOnElement(settingsButton);
  }

  // Multiremote functions
  async switchToOtherUserWindow() {
    const currentInstance = await driver.getWindowHandle();
    await driver.switchToWindow(currentInstance);
  }

  async showUplinkWindow() {
    const window = await this.window;
    await window.click();
    await window.waitForExist();
  }

  // Get Toggle Value

  async getToggleState(element: WebdriverIO.Element) {
    const currentDriver = await this.getCurrentDriver();
    let toggleState: string;
    let attributeToValidate: string;
    if (currentDriver === MACOS_DRIVER) {
      attributeToValidate = "value";
    } else {
      attributeToValidate = "Toggle.ToggleState";
    }
    toggleState = await element.getAttribute(attributeToValidate);
    return toggleState;
  }

  async validateToggleIsEnabled(element: WebdriverIO.Element) {
    const currentDriver = await this.getCurrentDriver();
    let attributeToValidate: string;
    if (currentDriver === MACOS_DRIVER) {
      attributeToValidate = "value";
    } else {
      attributeToValidate = "Toggle.ToggleState";
    }
    await element.waitUntil(
      async function () {
        return (await this.getAttribute(attributeToValidate)) === "1";
      },
      {
        timeout: 5000,
        timeoutMsg: "Expected toggle to be enabled",
      },
    );
  }

  async validateToggleIsDisabled(element: WebdriverIO.Element) {
    const currentDriver = await this.getCurrentDriver();
    let attributeToValidate: string;
    if (currentDriver === MACOS_DRIVER) {
      attributeToValidate = "value";
    } else {
      attributeToValidate = "Toggle.ToggleState";
    }
    await element.waitUntil(
      async function () {
        return (await this.getAttribute(attributeToValidate)) === "0";
      },
      {
        timeout: 5000,
        timeoutMsg: "Expected toggle to be disabled",
      },
    );
  }

  async validateNoModalIsOpen() {
    await this.modal.waitForExist({
      reverse: true,
    });
  }

  async clickOnOpenWebInspector() {
    const webInspectorOption = await this.contextMenuOpenWebInspector;
    await webInspectorOption.click();
  }

  async validateSpinnerIsNotShown(timeout: number = 60000) {
    await this.loaderSpinner.waitForExist({
      reverse: true,
      timeout: timeout,
    });
  }
}
