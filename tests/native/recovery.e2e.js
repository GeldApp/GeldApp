import assert from 'node:assert/strict';

const passphrase = 'synthetic e2e passphrase';
const persistedNote = 'isolated native E2E save';

describe('native vault recovery', () => {
  it('rejects a wrong recovery passphrase and restores the previous snapshot', async () => {
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
    assert.match(await error.getText(), /Previous vault authentication failed|Could not unlock previous vault/i);
    assert.ok((await $('h1').getText()).includes('Welcome back.'), 'failed recovery must remain locked');

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
  });
});
