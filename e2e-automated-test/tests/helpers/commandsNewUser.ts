import CreatePinScreen from "@screenobjects/account-creation/CreatePinScreen";
import CreateOrImportScreen from "@screenobjects/account-creation/CreateOrImportScreen";
import CreateUserScreen from "@screenobjects/account-creation/CreateUserScreen";
import SaveRecoverySeedScreen from "@screenobjects/account-creation/SaveRecoverySeedScreen";
import WelcomeScreen from "@screenobjects/welcome-screen/WelcomeScreen";
import { saveUserRecoverySeed } from "./commands";

export async function createNewUser(
  username: string,
  saveSeedWords: boolean = false,
) {
  await CreatePinScreen.unlockLayout.waitForExist();

  await CreatePinScreen.enterPinOnCreateAccount("1234");
  await CreatePinScreen.waitUntilCreateAccountButtonIsEnabled();
  await CreatePinScreen.clickOnCreateAccount();

  await CreateOrImportScreen.waitForIsShown(true);
  await CreateOrImportScreen.clickOnCreateAccount();

  await CreateUserScreen.waitForIsShown(true);
  await CreateUserScreen.enterUsername(username);
  await CreateUserScreen.waitUntilCreateAccountButtonIsEnabled();
  await CreateUserScreen.clickOnCreateAccount();

  await SaveRecoverySeedScreen.waitForIsShown(true);
  if (saveSeedWords === true) {
    const recoverySeed = await SaveRecoverySeedScreen.getSeedWords();
    await saveUserRecoverySeed(username, recoverySeed);
  }
  await SaveRecoverySeedScreen.clickOnISavedItButton();

  await WelcomeScreen.welcomeLayout.waitForExist({ timeout: 60000 });
}
