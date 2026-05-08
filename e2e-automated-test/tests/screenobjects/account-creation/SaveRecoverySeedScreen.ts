require("module-alias/register");
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";
import { WINDOWS_DRIVER } from "@helpers/constants";
import { selectorContainer } from "@screenobjects/AppScreen";

let SELECTORS: selectorContainer = {};

const SELECTORS_COMMON: selectorContainer = {};

const SELECTORS_WINDOWS: selectorContainer = {
  COPY_SEED_BUTTON: '[name="copy-seed-button"]',
  COPY_SEED_HELPER_TEXT:
    '//Group/Text[contains(@Name, "Write these words down")]',
  COPY_SEED_WORDS_LABEL: '[name="copy-seed-words"]',
  COPY_SEED_WORDS_LABEL_TEXT: '//Text[@Name="RECOVERY SEED"]',
  COPY_SEED_WORDS_LAYOUT: '[name="copy-seed-words-layout"]',
  GO_BACK_BUTTON: '[name="back-button"]',
  I_SAVED_IT_BUTTON: '[name="i-saved-it-button"]',
  SEED_WORD_VALUE_TEXT: "<Text>",
};

const SELECTORS_MACOS: selectorContainer = {
  COPY_SEED_BUTTON: "~copy-seed-button",
  COPY_SEED_HELPER_TEXT:
    '-ios class chain:**/XCUIElementTypeGroup/XCUIElementTypeStaticText[`value CONTAINS[cd] "Write these words down"`]',
  COPY_SEED_WORDS_LABEL: "~copy-seed-words",
  COPY_SEED_WORDS_LABEL_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  COPY_SEED_WORDS_LAYOUT: "~copy-seed-words-layout",
  GO_BACK_BUTTON: "~back-button",
  I_SAVED_IT_BUTTON: "~i-saved-it-button",
  SEED_WORD_VALUE_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

class SaveRecoverySeedScreen extends UplinkMainScreen {
  constructor() {
    super(SELECTORS.COPY_SEED_WORDS_LAYOUT);
  }

  public get copySeedButton() {
    return this.copySeedsWordsLayout.$(SELECTORS.COPY_SEED_BUTTON);
  }

  public get copySeedHelperText() {
    return this.copySeedsWordsLayout.$(SELECTORS.COPY_SEED_HELPER_TEXT);
  }

  public get copySeedWordsLabel() {
    return this.copySeedsWordsLayout.$(SELECTORS.COPY_SEED_WORDS_LABEL);
  }

  public get copySeedWordsLabelText() {
    return this.copySeedWordsLabel.$(SELECTORS.COPY_SEED_WORDS_LABEL_TEXT);
  }

  public get copySeedsWordsLayout() {
    return $(SELECTORS.COPY_SEED_WORDS_LAYOUT);
  }

  public get goBackButton() {
    return this.copySeedsWordsLayout.$(SELECTORS.GO_BACK_BUTTON);
  }

  public get iSavedItButton() {
    return this.copySeedsWordsLayout.$(SELECTORS.I_SAVED_IT_BUTTON);
  }

  async clickOnCopySeedButton() {
    const copySeedButton = await this.copySeedButton;
    await copySeedButton.click();
  }

  async clickOnGoBackButton() {
    const goBackButton = await this.goBackButton;
    await goBackButton.click();
  }

  async clickOnISavedItButton() {
    const iSavedItButton = await this.iSavedItButton;
    await iSavedItButton.click();
  }

  async getSeedWord(numberOfWord: string) {
    const currentDriver = await this.getCurrentDriver();
    let locatorOfWord: string = "";
    let word: string = "";
    if (currentDriver === WINDOWS_DRIVER) {
      locatorOfWord = '[name="seed-word-value-' + numberOfWord + '"]';
    } else {
      locatorOfWord = "~seed-word-value-" + numberOfWord;
    }
    word = await $(locatorOfWord).$(SELECTORS.SEED_WORD_VALUE_TEXT).getText();
    return word;
  }

  async getSeedWords() {
    let seedWords: string[] = [];
    for (let i = 1; i <= 12; i++) {
      seedWords.push(await this.getSeedWord(i.toString()));
    }
    return seedWords;
  }
}

export default new SaveRecoverySeedScreen();
