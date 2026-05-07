require("module-alias/register");
import { faker } from "@faker-js/faker";
import { MACOS_DRIVER, WINDOWS_DRIVER } from "@helpers/constants";
import {
  keyboardShiftEnter,
  keyboardShortcutPaste,
  getUplinkWindowHandle,
  selectFileOnMacos,
  selectFileOnWindows,
  keyboardShortcutSelectAll,
} from "@helpers/commands";
import UplinkMainScreen from "@screenobjects/UplinkMainScreen";
import { selectorContainer } from "@screenobjects/AppScreen";
const robot = require("robotjs");

let SELECTORS: selectorContainer = {};

const SELECTORS_COMMON: selectorContainer = {
  CHAT_LAYOUT: "~chat-layout",
};

const SELECTORS_WINDOWS: selectorContainer = {
  EDIT_MESSAGE_INPUT: '[name="edit-message-input"]',
  EMOJI_BUTTON: '[name="send-emoji-button"]',
  INPUT_CHAR_COUNTER: '[name="input-char-counter"]',
  INPUT_CHAR_COUNTER_TEXT: "<Text>",
  INPUT_CHAR_MAX_TEXT: '//Group[@Name="input-group"]/Group[4]/Text',
  INPUT_GROUP: '[name="input-group"]',
  INPUT_TEXT: "<Edit>",
  SEND_MESSAGE_BUTTON: '[name="send-message-button"]',
  TOOLTIP: '[name="tooltip"]',
  TOOLTIP_TEXT: "//Group/Text",
  UPLOAD_BUTTON: '[name="upload-button"]',
  UPLOAD_BUTTON_LOCAL_DISK: '[name="attach-files-from-local-disk-into-chat"]',
  UPLOAD_BUTTON_STORAGE: '[name="attach-files-from-storage-into-chat"]',
};

const SELECTORS_MACOS: selectorContainer = {
  EDIT_MESSAGE_INPUT: "~edit-message-input",
  EMOJI_BUTTON: "~send-emoji-button",
  INPUT_CHAR_COUNTER: "~input-char-counter",
  INPUT_CHAR_COUNTER_TEXT: "-ios class chain:**/XCUIElementTypeStaticText",
  INPUT_CHAR_MAX_TEXT:
    '-ios class chain:**/XCUIElementTypeGroup[`label == "input-group"`]/XCUIElementTypeGroup[4]/XCUIElementTypeStaticText',
  INPUT_GROUP: "~input-group",
  INPUT_TEXT: "-ios class chain:**/XCUIElementTypeTextView",
  SEND_MESSAGE_BUTTON: "~send-message-button",
  TOOLTIP: "~tooltip",
  TOOLTIP_TEXT:
    "-ios class chain:**/XCUIElementTypeGroup/XCUIElementTypeStaticText",
  UPLOAD_BUTTON: "~upload-button",
  UPLOAD_BUTTON_LOCAL_DISK: "~attach-files-from-local-disk-into-chat",
  UPLOAD_BUTTON_STORAGE: "~attach-files-from-storage-into-chat",
};

process.env.DRIVER === WINDOWS_DRIVER
  ? (SELECTORS = { ...SELECTORS_WINDOWS, ...SELECTORS_COMMON })
  : (SELECTORS = { ...SELECTORS_MACOS, ...SELECTORS_COMMON });

class InputBar extends UplinkMainScreen {
  constructor() {
    super(SELECTORS.CHAT_LAYOUT);
  }

  public get editMessageInput() {
    return $(SELECTORS.EDIT_MESSAGE_INPUT);
  }

  public get emojiButton() {
    return $(SELECTORS.EMOJI_BUTTON);
  }

  public get inputCharCounter() {
    return $(SELECTORS.INPUT_CHAR_COUNTER);
  }

  public get inputCharCounterText() {
    return $(SELECTORS.INPUT_CHAR_COUNTER).$(SELECTORS.INPUT_CHAR_COUNTER_TEXT);
  }

  public get inputCharMaxText() {
    return $(SELECTORS.INPUT_CHAR_MAX_TEXT);
  }

  public get inputGroup() {
    return $(SELECTORS.CHAT_LAYOUT).$(SELECTORS.INPUT_GROUP);
  }

  public get inputText() {
    return $(SELECTORS.CHAT_LAYOUT)
      .$(SELECTORS.INPUT_GROUP)
      .$(SELECTORS.INPUT_TEXT);
  }

  public get sendMessageButton() {
    return $(SELECTORS.SEND_MESSAGE_BUTTON);
  }

  public get sendMessageTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get sendMessageTooltipText() {
    return $(SELECTORS.TOOLTIP).$(SELECTORS.TOOLTIP_TEXT);
  }

  public get uploadButton() {
    return $(SELECTORS.UPLOAD_BUTTON);
  }

  public get uploadTooltip() {
    return $(SELECTORS.TOOLTIP);
  }

  public get uploadTooltipText() {
    return $(SELECTORS.TOOLTIP).$(SELECTORS.TOOLTIP_TEXT);
  }

  public get uploadButtonStorage() {
    return $(SELECTORS.UPLOAD_BUTTON_STORAGE);
  }

  public get uploadButtonLocalDisk() {
    return $(SELECTORS.UPLOAD_BUTTON_LOCAL_DISK);
  }

  async clearInputBar() {
    const inputText = await this.inputText;
    await inputText.clearValue();
  }

  async clearLongInputBar() {
    const inputText = await this.inputText;
    await inputText.click();
    await keyboardShortcutSelectAll();
    await this.typeMessageOnInput("a");
    await inputText.clearValue();
  }

  async clickOnEmojiButton() {
    const addEmoji = await this.emojiButton;
    await this.hoverOnElement(addEmoji);
    await addEmoji.click();
  }

  async clickOnInputBar() {
    const inputText = await this.inputText;
    await inputText.click();
  }

  async clickOnSendMessage() {
    const sendMessageButton = await this.sendMessageButton;
    await sendMessageButton.click();
  }

  async clickOnUploadFile() {
    const uploadButton = await this.uploadButton;
    await uploadButton.click();
    const uploadButtonStorage = await this.uploadButtonStorage;
    await uploadButtonStorage.waitForExist();
  }

  async generateRandomText() {
    // public get a random word of 9 chars and add it a space
    const wordToRepeat = faker.lorem.word(9) + " ";
    // Then repeat the same word for 102 times (1020 chars)
    let longParagraph = wordToRepeat.repeat(102);
    // Now, add 4 more chars, to have 1024 chars
    longParagraph += "abcd";
    return longParagraph;
  }

  async generateShortRandomText() {
    // public get a random word of 9 chars and add it a space
    const wordToRepeat = faker.lorem.word(9) + " ";
    // Then repeat the same word for 10 times
    const longText = wordToRepeat.repeat(10);
    return longText;
  }

  async getValueFromInputBar() {
    const inputText = await this.inputText;
    let inputTextValue = (await inputText.getText()).slice(0, -1);
    return inputTextValue;
  }

  async hoverOnSendButton() {
    const sendButton = await this.sendMessageButton;
    await this.hoverOnElement(sendButton);
  }

  async hoverOnUploadButton() {
    const uploadButton = await this.uploadButton;
    await this.hoverOnElement(uploadButton);
  }

  async pasteClipboardOnInputBar() {
    const inputText = await this.inputText;
    await inputText.click();
    await inputText.clearValue();
    await keyboardShortcutPaste();
  }

  async pressEnterKeyOnInputBar() {
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === WINDOWS_DRIVER) {
      const inputText = await this.inputText;
      await inputText.addValue("\uE007");
    } else {
      await this.clickOnSendMessage();
    }
  }

  async selectUploadFromLocalDisk() {
    const uploadButtonLocalDisk = await this.uploadButtonLocalDisk;
    await uploadButtonLocalDisk.waitForExist();
    await uploadButtonLocalDisk.click();
  }

  async selectUploadFromStorage() {
    const uploadButtonStorage = await this.uploadButtonStorage;
    await uploadButtonStorage.waitForExist();
    await uploadButtonStorage.click();
  }

  async typeCodeOnInputBar(language: string, codeToType: string) {
    const inputText = await this.inputText;
    await inputText.clearValue();
    await inputText.addValue("```" + language);
    let inputTextValueLanguage = await inputText.getText();
    if (inputTextValueLanguage.includes("```" + language) === false) {
      await this.typeCodeOnInputBar(language, codeToType);
    } else {
      await keyboardShiftEnter();
      await robot.typeString(codeToType);
      let inputTextValueCode = await inputText.getText();
      inputTextValueCode += " ";
      if (inputTextValueCode.includes(codeToType) === false) {
        await this.typeCodeOnInputBar(language, codeToType);
      }
    }
  }

  async typeMessageOnInput(text: string) {
    const inputText = await this.inputText;
    await inputText.clearValue();
    await inputText.setValue(text);
    const inputTextElement = await this.inputText;
    let inputTextElementValue = await inputTextElement.getText();
    if (inputTextElementValue.includes(text) === false) {
      await this.typeMessageOnInput(text);
    }
  }

  async uploadFileFromLocalDisk(relativePath: string) {
    const currentDriver = await this.getCurrentDriver();
    if (currentDriver === MACOS_DRIVER) {
      await this.clickOnUploadFile();
      await this.selectUploadFromLocalDisk();
      await selectFileOnMacos(relativePath);
    } else {
      const uplinkContext = await getUplinkWindowHandle();
      await this.clickOnUploadFile();
      await this.selectUploadFromLocalDisk();
      await selectFileOnWindows(relativePath, uplinkContext);
    }
  }
}

export default new InputBar();
