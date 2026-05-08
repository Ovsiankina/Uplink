require("module-alias/register");
const allureReporter = require("@wdio/allure-reporter").default;
const sharedConfig = require("@config/wdio.shared.conf.ts").config;
const homedir = require("os").homedir;
const join = require("path").join;
const fsp = require("fs").promises;
const kill = require("kill-port");

const { readFileSync, rmSync } = require("fs");
const WINDOWS_APP_LOCATION = require("@helpers/constants").WINDOWS_APP;

export const config: WebdriverIO.Config = {
  ...sharedConfig,
  ...{
    port: 4723,
    path: "/wd/hub/",
    specs: [
      join(
        process.cwd(),
        "./tests/suites/Chats/02-WindowsMessages.suite.ts",
      ),
    ],
    exclude: [],
    capabilities: [
      {
        platformName: "windows",
        "appium:deviceName": "WindowsPC",
        "appium:automationName": "windows",
        "appium:app": WINDOWS_APP_LOCATION,
        // --with_mock makes Uplink load mock state (20 friends + chats)
        //   instead of going to the network, so we can open a chat and send
        //   a message with no second instance and no Warp shuttle.
        // --discovery disable stops Uplink from blocking on the dead public
        //   Shuttle peer during init.
        "appium:appArguments": "--with-mock --discovery disable",
        "ms:waitForAppLaunch": 60,
      },
    ],
    reporters: [
      ["spec", { showPreface: false }],
      [
        "allure",
        {
          outputDir: "./allure-results",
          disableWebdriverStepsReporting: true,
          disableWebdriverScreenshotsReporting: true,
        },
      ],
    ],
    specFileRetries: 0, // no retries — setup must succeed first time
    onComplete: async function () {
      await kill(4723, "tcp");
      // Kill any leftover uplink processes (including orphans from a failed
      // WinAppDriver session attempt that don't exit on their own).
      try {
        const { execSync } = require("child_process");
        execSync("taskkill /IM uplink.exe /F", { stdio: "ignore" });
      } catch (e) {}
    },
    onPrepare: async function () {
      // Clean both Uplink data dirs so we start fresh
      const dirs = ["/.uplink", "/.uplinkUserB"];
      for (const dir of dirs) {
        const fullPath = homedir() + dir + "/.user";
        try {
          await rmSync(fullPath, { recursive: true, force: true });
        } catch (e) {}
      }
      // Clean report dirs
      for (const d of ["allure-results", "test-report", "test-results"]) {
        try {
          await rmSync(join(process.cwd(), d), { recursive: true, force: true });
        } catch (e) {}
      }
    },
    afterTest: async function (
      test,
      context,
      { error, result, duration, passed, retries },
    ) {
      if (!passed) {
        try {
          let imageFile = await driver.takeScreenshot();
          const imageFolder = join(
            process.cwd(),
            "./test-results/windows-chats",
            test.parent,
          );
          const imageTitle = test.title + " - Failed.png";
          await fsp.mkdir(imageFolder, { recursive: true });
          await fsp.writeFile(imageFolder + "/" + imageTitle, imageFile, "base64");
          const data = await readFileSync(`${imageFolder}/${imageTitle}`);
          allureReporter.addAttachment(imageTitle, data, "image/png");
        } catch (e) {}
      }
    },
  },
};
