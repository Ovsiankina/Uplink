require("module-alias/register");
export default class AppScreen {
  private locator;

  constructor(locator: string) {
    this.locator = locator;
  }

  async getCurrentDriver() {
    const currentDriver = process.env.DRIVER;
    return currentDriver;
  }

  async waitForIsShown(isShown = true): Promise<boolean | void> {
    const locator = await $(this.locator);
    return locator.waitForDisplayed({
      reverse: !isShown,
    });
  }
}

export type selectorContainer = {
  [key: string]: string;
};
