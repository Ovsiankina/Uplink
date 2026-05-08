require("module-alias/register");
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";
import { WINDOWS_DRIVER } from "@helpers/constants";
import { selectorContainer } from "@screenobjects/AppScreen";

const SELECTORS_COMMON: selectorContainer = {};

const SELECTORS_WINDOWS: selectorContainer = {
  CREATE_NEW_ACCOUNT_BUTTON: '[name="create-button"]',
  CREATE_OR_RECOVER_LABEL: '[name="create-or-recover"]',
  CREATE_OR_RECOVER_LABEL_TEXT: '//Text[@Name="ACCOUNT CREATION"]',
  IMPORT_ACCOUNT_BUTTON: '[name="recover-button"]',
  RECOVERY_LAYOUT: '[name="create-or-recover-layout"]',
  RECOVERY_PARAGRAPH: '[name="create-or-recover-instructions"]',
  RECOVERY_PARAGRAPH_TEXT: "<Text>",
};

const SELECTORS_MACOS: selectorContainer = {
  CREATE_NEW_ACCOUNT_BUTTON: "~create-button",
  CREATE_OR_RECOVER_LABEL: "~create-or-recover",
  CREATE_OR_RECOVER_LABEL_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  IMPORT_ACCOUNT_BUTTON: "~recover-button",
  RECOVERY_LAYOUT: "~create-or-recover-layout",
  RECOVERY_PARAGRAPH: "~create-or-recover-instructions",
  RECOVERY_PARAGRAPH_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
};

let SELECTORS: selectorContainer = {
  CREATE_NEW_ACCOUNT_BUTTON: "",
  CREATE_OR_RECOVER_LABEL: "",
  CREATE_OR_RECOVER_LABEL_TEXT: "",
  IMPORT_ACCOUNT_BUTTON: "",
  RECOVERY_LAYOUT: "",
  RECOVERY_PARAGRAPH: "",
  RECOVERY_PARAGRAPH_TEXT: "",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

class CreateOrImportScreen extends UplinkMainScreen {
  constructor() {
    super(SELECTORS.RECOVERY_LAYOUT);
  }

  public get createNewAccountButton() {
    return this.recoveryLayout.$(SELECTORS.CREATE_NEW_ACCOUNT_BUTTON);
  }

  public get createOrRecoverLabel() {
    return this.recoveryLayout.$(SELECTORS.CREATE_OR_RECOVER_LABEL);
  }

  public get createOrRecoverLabelText() {
    return this.createOrRecoverLabel.$(SELECTORS.CREATE_OR_RECOVER_LABEL_TEXT);
  }

  public get importAccountButton() {
    return this.recoveryLayout.$(SELECTORS.IMPORT_ACCOUNT_BUTTON);
  }

  public get recoveryLayout() {
    return $(SELECTORS.RECOVERY_LAYOUT);
  }

  public get recoveryParagraph() {
    return this.recoveryLayout.$(SELECTORS.RECOVERY_PARAGRAPH);
  }

  public get recoveryParagraphText() {
    return this.recoveryParagraph.$(SELECTORS.RECOVERY_PARAGRAPH_TEXT);
  }

  async clickOnCreateAccount() {
    const createAccountButton = await this.createNewAccountButton;
    await createAccountButton.click();
    await this.validateSpinnerIsNotShown();
  }

  async clickOnImportAccount() {
    const importAccountButton = await this.importAccountButton;
    await importAccountButton.click();
  }
}

export default new CreateOrImportScreen();
