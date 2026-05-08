require("module-alias/register");
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";
import { MACOS_DRIVER, WINDOWS_DRIVER } from "@helpers/constants";
import { selectorContainer } from "@screenobjects/AppScreen";

let SELECTORS: selectorContainer = {};

const SELECTORS_COMMON: selectorContainer = {
  UNLOCK_LAYOUT: "~unlock-layout",
  USERNAME_INPUT: "~username-input",
};

const SELECTORS_WINDOWS: selectorContainer = {
  CREATE_ACCOUNT_BUTTON: '[name="create-account-button"]',
  CREATE_USER_HELPER: '[name="instructions"]',
  CREATE_USER_HELPER_TEXT: "<Text>",
  CREATE_USER_LABEL_TEXT: '//Text[@Name="ENTER USERNAME"]',
  INPUT_ERROR: '[name="input-error"]',
  INPUT_ERROR_TEXT: "<Text>",
};

const SELECTORS_MACOS: selectorContainer = {
  CREATE_ACCOUNT_BUTTON: "~create-account-button",
  CREATE_USER_HELPER: "~instructions",
  CREATE_USER_HELPER_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  CREATE_USER_LABEL_TEXT:
    "-ios class chain:**/XCUIElementTypeStaticText/XCUIElementTypeStaticText",
  INPUT_ERROR: "~input-error",
  INPUT_ERROR_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

class CreateUserScreen extends UplinkMainScreen {
  constructor() {
    super(SELECTORS.UNLOCK_LAYOUT);
  }

  public get createAccountButton() {
    return this.unlockLayout.$(SELECTORS.CREATE_ACCOUNT_BUTTON);
  }

  public get createUserHelper() {
    return this.unlockLayout.$(SELECTORS.CREATE_USER_HELPER);
  }

  public get createUserHelperText() {
    return this.createUserHelper.$(SELECTORS.CREATE_USER_HELPER_TEXT);
  }

  public get createUserLabelText() {
    return this.unlockLayout.$(SELECTORS.CREATE_USER_LABEL_TEXT);
  }

  public get inputError() {
    return this.unlockLayout.$(SELECTORS.INPUT_ERROR);
  }

  public get inputErrorText() {
    return this.inputError.$(SELECTORS.INPUT_ERROR_TEXT);
  }

  public get unlockLayout() {
    return $(SELECTORS.UNLOCK_LAYOUT);
  }

  public get usernameInput() {
    return this.unlockLayout.$(SELECTORS.USERNAME_INPUT);
  }

  async enterUsername(username: string) {
    const currentDriver = await this.getCurrentDriver();
    const usernameInput = await this.usernameInput;
    await usernameInput.clearValue();
    await usernameInput.setValue(username);
    if (currentDriver === MACOS_DRIVER) {
      const usernameInputValue = await usernameInput.getValue();
      if (usernameInputValue !== username) {
        await this.enterUsername(username);
      }
    }
  }

  async clickOnCreateAccount() {
    const createAccountButton = await this.createAccountButton;
    await createAccountButton.click();
    await this.validateSpinnerIsNotShown();
  }

  async getStatusOfCreateAccountButton() {
    let result;
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === WINDOWS_DRIVER) {
      result = await this.createAccountButton.getAttribute("IsEnabled");
    } else {
      result = await this.createAccountButton.getAttribute("enabled");
    }
    return result.toString().toLowerCase();
  }

  async waitUntilCreateAccountButtonIsEnabled() {
    const createAccountButton = await this.createAccountButton;
    await driver.waitUntil(
      async () => {
        return await createAccountButton.waitForEnabled();
      },
      {
        timeout: 30000,
        timeoutMsg: "Expected status was not changed to enabled after 30s",
      },
    );
  }
}

export default new CreateUserScreen();
