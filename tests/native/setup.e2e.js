import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const passphrase = 'synthetic e2e passphrase';
const persistedNote = 'isolated native E2E save';

describe('native vault setup', () => {
  it('creates, saves, and locks a synthetic vault in the compiled app', async () => {
    await $('h1').waitForDisplayed({ timeout: 30_000 });
    const heading = await $('h1').getText();
    console.log('Native app initial screen:', heading, (await $('body').getText()).slice(0, 500));
    await browser.waitUntil(async () => (await $('h1').getText()).includes('Create your private vault.'), {
      timeout: 30_000,
      timeoutMsg: 'The compiled app did not reach the isolated vault setup screen.',
    });

    const passphraseFields = await $$('input[type="password"]');
    assert.equal(passphraseFields.length, 2, (await browser.getPageSource()).slice(0, 4_000));
    await passphraseFields[0].setValue(passphrase);
    await passphraseFields[1].setValue(passphrase);
    await $('button*=Create encrypted vault').click();

    await browser.waitUntil(async () => (await $('h1').getText()).includes('A little more clarity.'), {
      timeout: 60_000,
      timeoutMsg: 'The compiled app did not unlock after native vault creation.',
    });

    const appBinary = path.resolve('src-tauri/target/debug', process.platform === 'win32' ? 'geldapp.exe' : 'geldapp');
    const secondProcess = spawn(appBinary, [], {
      stdio: 'ignore',
      env: { ...process.env, TAURI_WEBDRIVER_PORT: '4446' },
    });
    const exited = new Promise((resolve) => secondProcess.once('exit', (code, signal) => resolve({ code, signal })));
    const processResult = await Promise.race([
      exited,
      delay(10_000).then(() => null),
    ]);
    if (processResult === null) {
      secondProcess.kill();
      await exited;
    }
    assert.notEqual(processResult, null, 'a second app process must not open the active vault');
    assert.notEqual(processResult.code, 0, 'a second app process must fail to acquire the vault lock');
    await $('button*=Transactions').click();
    await $('button*=REWE').click();
    await $('textarea').setValue(persistedNote);
    await $('button=Done').click();

    await $('button*=Settings').click();
    await $('button*=Lock vault').click();
    await browser.waitUntil(async () => (await $('h1').getText()).includes('Welcome back.'), {
      timeout: 30_000,
      timeoutMsg: 'The app did not save and lock through the native vault flow.',
    });
  });
});
