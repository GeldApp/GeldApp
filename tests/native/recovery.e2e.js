import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const passphrase = 'synthetic e2e passphrase';
const persistedNote = 'isolated native E2E save';

describe('native vault recovery', () => {
  it('rejects a wrong recovery passphrase and restores the previous snapshot', async () => {
    const dataDirectory = process.env.GELDAPP_E2E_DATA_DIR;
    const currentPath = path.join(dataDirectory, 'vault.enc');
    const previousPath = path.join(dataDirectory, 'vault.previous.enc');
    const corruptedCurrent = fs.readFileSync(currentPath);
    const originalPrevious = fs.readFileSync(previousPath);
    assert.equal(corruptedCurrent.toString(), 'synthetic deliberate corruption');

    await browser.waitUntil(async () => (await $('h1').getText()).includes('Welcome back.'), {
      timeout: 30_000,
      timeoutMsg: 'The app did not offer passphrase recovery after detecting the damaged current snapshot.',
    });
    const recovery = await $('.vault-recovery');
    await recovery.waitForDisplayed({ timeout: 15_000 });
    await recovery.$('input[type="checkbox"]').click();
    let passphraseField = await $('input[type="password"]');
    await passphraseField.setValue('deliberately wrong recovery passphrase');
    await recovery.$('button*=Recover previous snapshot').click();
    const error = await $('p[role="alert"]');
    await error.waitForDisplayed({ timeout: 30_000 });
    assert.match(await error.getText(), /Could not recover previous vault/i);
    assert.ok((await $('h1').getText()).includes('Welcome back.'), 'failed recovery must remain locked');
    assert.deepEqual(fs.readFileSync(currentPath), corruptedCurrent, 'failed recovery must preserve the damaged current file');
    assert.deepEqual(fs.readFileSync(previousPath), originalPrevious, 'failed recovery must preserve the previous snapshot');

    passphraseField = await $('input[type="password"]');
    await passphraseField.setValue(passphrase);
    await recovery.$('button*=Recover previous snapshot').click();
    await browser.waitUntil(async () => (await $('h1').getText()).includes('A little more clarity.'), {
      timeout: 60_000,
      timeoutMsg: 'The correct passphrase did not restore the previous encrypted snapshot.',
    });
    await $('button*=Transactions').click();
    await $('button*=REWE').click();
    assert.equal(await $('.modal p').getText(), persistedNote, 'recovery did not restore the saved synthetic transaction note');

    const rejectedFiles = fs.readdirSync(dataDirectory).filter((name) => name.startsWith('vault.rejected-'));
    assert.equal(rejectedFiles.length, 1, 'successful recovery must preserve the rejected current snapshot');
    assert.deepEqual(fs.readFileSync(path.join(dataDirectory, rejectedFiles[0])), corruptedCurrent);
    assert.deepEqual(fs.readFileSync(previousPath), originalPrevious, 'recovery must keep the previous snapshot intact');
    assert.notDeepEqual(fs.readFileSync(currentPath), corruptedCurrent, 'recovery must replace the corrupted current file');
  });
});
