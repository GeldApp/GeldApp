import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const passphrase = 'synthetic e2e passphrase';
const persistedNote = 'isolated native E2E save';

describe('native vault restart', () => {
  it('rejects a wrong passphrase, then reopens the saved ledger', async () => {
    await browser.waitUntil(async () => (await $('h1').getText()).includes('Welcome back.'), {
      timeout: 30_000,
      timeoutMsg: 'The persisted test vault did not survive app restart.',
    });

    let field = await $('input[type="password"]');
    await field.setValue('deliberately wrong synthetic passphrase');
    await $('button*=Unlock vault').click();
    const error = await $('p[role="alert"]');
    await error.waitForDisplayed({ timeout: 30_000 });
    assert.match(await error.getText(), /Could not unlock vault|authentication failed/i);
    assert.ok((await $('h1').getText()).includes('Welcome back.'), 'wrong passphrase must keep the app locked');

    field = await $('input[type="password"]');
    await field.setValue(passphrase);
    await $('button*=Unlock vault').click();
    await browser.waitUntil(async () => (await $('h1').getText()).includes('A little more clarity.'), {
      timeout: 60_000,
      timeoutMsg: 'The correct passphrase did not reopen the encrypted ledger.',
    });
    await $('button*=REWE').click();
    assert.equal(await $('.modal p').getText(), persistedNote, 'the changed synthetic note was not restored');

    await $('button=Done').click();
    await $('button*=Settings').click();
    await $('button*=Lock vault').click();
    await browser.waitUntil(async () => (await $('h1').getText()).includes('Welcome back.'), {
      timeout: 30_000,
      timeoutMsg: 'The app did not lock after its reopened save.',
    });
    fs.writeFileSync(path.join(process.env.GELDAPP_E2E_DATA_DIR, 'vault.enc'), 'synthetic deliberate corruption');
  });
});
