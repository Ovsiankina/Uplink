require("module-alias/register");
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";
import { WINDOWS_DRIVER } from "@helpers/constants";
import { selectorContainer } from "@screenobjects/AppScreen";

let SELECTORS: selectorContainer = {};

const SELECTORS_COMMON: selectorContainer = {};

const SELECTORS_WINDOWS: selectorContainer = {
  ADD_FRIENDS_BUTTON: '[name="add-friends-button"]',
  ADD_SOMEONE_TEXT: "<Text>",
  WELCOME_IMAGE: '[name="welcome-image"]',
  WELCOME_LAYOUT: "~welcome",
};

const SELECTORS_MACOS: selectorContainer = {
  ADD_FRIENDS_BUTTON: "~add-friends-button",
  ADD_SOMEONE_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  WELCOME_IMAGE: "~welcome-image",
  WELCOME_LAYOUT: "~welcome-screen",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

class WelcomeScreen extends UplinkMainScreen {
  constructor() {
    super(SELECTORS.WELCOME_LAYOUT);
  }

  public get addFriendsButton() {
    return this.welcomeLayout.$(SELECTORS.ADD_FRIENDS_BUTTON);
  }

  public get addSomeoneText() {
    return this.welcomeLayout.$(SELECTORS.ADD_SOMEONE_TEXT);
  }

  public get welcomeImage() {
    return this.welcomeLayout.$(SELECTORS.WELCOME_IMAGE);
  }

  public get welcomeLayout() {
    return $(SELECTORS.WELCOME_LAYOUT);
  }

  async clickAddSomeone() {
    const addSomeoneButton = await this.addFriendsButton;
    await addSomeoneButton.click();
  }

  async validateWelcomeScreenIsShown(timeout: number = 15000) {
    await this.welcomeLayout.waitForExist({ timeout: timeout });
  }
}

export default new WelcomeScreen();
