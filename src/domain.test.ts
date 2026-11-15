import { describe, it, expect } from 'vitest';
import { parseMoney, validIban, parseCsv, exportCsv, importTransactions, validateLedger } from './domain';
import type { Transaction } from './domain';
import { demoLedger } from './demo';
import { encryptBackup, decryptBackup } from './storage';

describe('money and account validation',()=>{
 it('keeps exact minor units and rejects ambiguous amounts',()=>{expect(parseMoney('19.99')).toBe(1999);expect(parseMoney('-0.01')).toBe(-1);expect(()=>parseMoney('1,200')).toThrow();expect(()=>parseMoney('0.001')).toThrow();});
 it('checks IBAN checksum',()=>{expect(validIban('DE89 3704 0044 0532 0130 00')).toBe(true);expect(validIban('DE88 3704 0044 0532 0130 00')).toBe(false);});
});
describe('CSV import',()=>{
 it('round-trips commas, quotes, line breaks and Unicode',()=>{const tx={...demoLedger().transactions[0],merchant:'Café, "North"',note:'hello\nworld'};const result=parseCsv(exportCsv([tx]),tx.accountId);expect(result[0]).toMatchObject({merchant:tx.merchant,note:tx.note,amount:tx.amount,date:tx.date,category:tx.category});});
 it('prefixes formula-like text fields and keeps signed amounts numeric',()=>{
  const tx:Transaction={...demoLedger().transactions[0],merchant:'  =1+1',category:'\t+SUM(1,1)' as unknown as Transaction['category'],note:'\u0000@SUM("A",1)',amount:-1234};
  const csv=exportCsv([tx]);
  expect(csv).toContain('"\'  =1+1"');
  expect(csv).toContain('"\'\t+SUM(1,1)"');
  expect(csv).toContain('"\'\u0000@SUM(""A"",1)"');
  expect(csv.split('\r\n')[1]).toContain(',-12.34,');
  expect(exportCsv([{...tx,merchant:' -1+1'}])).toContain('"\' -1+1"');
  const imported=parseCsv(csv,tx.accountId)[0];
  expect(imported).toMatchObject({merchant:"'  =1+1",category:'Other',note:"'\u0000@SUM(\"A\",1)",amount:-1234});
 });
 it('deduplicates imports and updates only matching account balance',()=>{const ledger=demoLedger();const tx=parseCsv('date,merchant,amount\n2026-11-15,New merchant,-12.34','n26');const first=importTransactions(ledger,[...tx,...tx]);expect(first.added).toBe(1);expect(first.ledger.accounts[0].balance).toBe(ledger.accounts[0].balance-1234);expect(first.ledger.accounts[1]).toEqual(ledger.accounts[1]);expect(importTransactions(first.ledger,tx).added).toBe(0);});
 it('rejects invalid calendar dates and malformed CSV',()=>{expect(()=>parseCsv('date,merchant,amount\n2026-02-30,Test,1','n26')).toThrow();expect(()=>parseCsv('date,merchant,amount\n2026-01-01,"bad,1','n26')).toThrow();});
 it('rejects orphan transaction data and fractional minor units',()=>{const data=demoLedger();data.transactions[0].accountId='missing';expect(()=>validateLedger(data)).toThrow();const other=demoLedger();other.accounts[0].balance=.5;expect(()=>validateLedger(other)).toThrow();});
});
describe('encrypted backup',()=>{
 it('round-trips a ledger with random salt and IV',async()=>{const ledger=demoLedger();const password='a strong test passphrase';const one=await encryptBackup(ledger,password),two=await encryptBackup(ledger,password);expect(one).not.toEqual(two);expect(one).not.toContain('REWE');expect(await decryptBackup(one,password)).toEqual(ledger);});
 it('rejects a wrong key and tampered ciphertext',async()=>{const raw=await encryptBackup(demoLedger(),'a strong test passphrase');await expect(decryptBackup(raw,'wrong password')).rejects.toThrow('Could not unlock');const envelope=JSON.parse(raw);envelope.ciphertext=(envelope.ciphertext[0]==='A'?'B':'A')+envelope.ciphertext.slice(1);await expect(decryptBackup(JSON.stringify(envelope),'a strong test passphrase')).rejects.toThrow();});
});
