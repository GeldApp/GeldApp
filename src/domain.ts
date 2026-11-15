export const categories = ['Groceries', 'Food & drink', 'Shopping', 'Transport', 'Home', 'Subscriptions', 'Health', 'Income', 'Transfers', 'Other'] as const;
export type Category = typeof categories[number];
export type Account = { id: string; name: string; bank: string; iban: string; balance: number; color: string; kind: 'Current account' | 'Savings' | 'Cash' };
export type Transaction = { id: string; accountId: string; merchant: string; note: string; date: string; amount: number; category: Category };
export type Budget = { id: string; category: Category; limit: number };
export type Draft = { id: string; accountId: string; recipient: string; iban: string; amount: number; reference: string };
export type Ledger = { version: 1; accounts: Account[]; transactions: Transaction[]; budgets: Budget[]; drafts: Draft[] };
export const money = (cents: number, decimals = true) => new Intl.NumberFormat('en-IE', { style: 'currency', currency: 'EUR', minimumFractionDigits: decimals ? 2 : 0, maximumFractionDigits: decimals ? 2 : 0 }).format(cents / 100);
export const today = () => new Date().toLocaleDateString('en-CA');
export function parseMoney(value: string): number {
  if (!/^-?\d+(\.\d{1,2})?$/.test(value.trim())) throw new Error('Use a number such as 42.50 (up to two decimals).');
  const amount = Math.round(Number(value) * 100);
  if (!Number.isSafeInteger(amount)) throw new Error('Amount is too large.');
  return amount;
}
export function validIban(value: string) {
  const iban = value.replace(/\s/g, '').toUpperCase();
  if (!/^[A-Z]{2}\d{2}[A-Z0-9]{11,30}$/.test(iban)) return false;
  const digits = (iban.slice(4) + iban.slice(0, 4)).replace(/[A-Z]/g, c => String(c.charCodeAt(0) - 55));
  let remainder = 0;
  for (const digit of digits) remainder = (remainder * 10 + Number(digit)) % 97;
  return remainder === 1;
}
export function validateLedger(input: unknown): Ledger {
  const data = input as Ledger;
  if (!data || data.version !== 1 || !Array.isArray(data.accounts) || !Array.isArray(data.transactions) || !Array.isArray(data.budgets) || !Array.isArray(data.drafts)) throw new Error('This is not a valid GeldApp backup.');
  const str = (v: unknown) => typeof v === 'string';
  const int = (v: unknown) => Number.isSafeInteger(v);
  if (data.accounts.some(a => !a || !str(a.id) || !str(a.name) || !str(a.bank) || !str(a.iban) || !str(a.color) || !int(a.balance) || !['Current account','Savings','Cash'].includes(a.kind))) throw new Error('Invalid account data.');
  const ids = new Set(data.accounts.map(a => a.id));
  if (ids.size !== data.accounts.length) throw new Error('Duplicate account IDs.');
  if (data.transactions.some(t => !t || !str(t.id) || !ids.has(t.accountId) || !str(t.merchant) || !str(t.note) || !validDate(t.date) || !int(t.amount) || !categories.includes(t.category))) throw new Error('Invalid transaction data.');
  if (new Set(data.transactions.map(t=>t.id)).size !== data.transactions.length) throw new Error('Duplicate transaction IDs.');
  if (data.budgets.some(b => !b || !str(b.id) || !categories.includes(b.category) || !int(b.limit) || b.limit <= 0)) throw new Error('Invalid budget data.');
  if (data.drafts.some(d => !d || !str(d.id) || !ids.has(d.accountId) || !str(d.recipient) || !str(d.iban) || !int(d.amount) || d.amount <= 0 || !str(d.reference))) throw new Error('Invalid payment draft.');
  return data;
}
export function validDate(value: unknown): value is string {
  return typeof value === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(value) && !Number.isNaN(Date.parse(value)) && new Date(value).toISOString().slice(0,10) === value;
}
const escapeCsv = (s: string) => '"' + s.replaceAll('"', '""') + '"';
const safeCsvText = (s: string) => {
  // Spreadsheet applications may trim leading whitespace/control characters
  // before interpreting a cell. Prefix formula-like text before CSV quoting.
  return escapeCsv(/^[\s\p{Cc}]*[=+\-@]/u.test(s) ? `'${s}` : s);
};
export function exportCsv(transactions: Transaction[]) {
  return ['date,merchant,amount,category,note', ...transactions.map(t => [safeCsvText(t.date), safeCsvText(t.merchant), (t.amount / 100).toFixed(2), safeCsvText(t.category), safeCsvText(t.note)].join(','))].join('\r\n');
}
export function parseCsv(text: string, accountId: string): Transaction[] {
  const rows: string[][] = []; let row: string[] = [], field = '', quoted = false;
  text = text.replace(/^\uFEFF/, '');
  for (let i=0; i<text.length; i++) {
    const c=text[i];
    if (c==='"') { if (quoted && text[i+1]==='"') { field+='"'; i++; } else quoted=!quoted; }
    else if (c===',' && !quoted) { row.push(field); field=''; }
    else if ((c==='\n' || c==='\r') && !quoted) { if (c==='\r' && text[i+1]==='\n') i++; row.push(field); if (row.some(Boolean)) rows.push(row); row=[]; field=''; }
    else field+=c;
  }
  if (quoted) throw new Error('Unclosed quote in CSV.');
  row.push(field); if (row.some(Boolean)) rows.push(row);
  const header=rows.shift()?.map(v=>v.trim().toLowerCase()) || [];
  for (const required of ['date','merchant','amount']) if (!header.includes(required)) throw new Error('CSV needs date, merchant, and amount columns.');
  if (rows.length === 0) throw new Error('No transactions found.');
  if (rows.length > 10000) throw new Error('Import up to 10,000 rows at a time.');
  return rows.map((r,index) => {
    const get=(key:string)=>r[header.indexOf(key)]?.trim() || '';
    if (!validDate(get('date')) || !get('merchant')) throw new Error(`Row ${index+2}: use YYYY-MM-DD and a merchant name.`);
    const category = categories.includes(get('category') as Category) ? get('category') as Category : 'Other';
    return {id:crypto.randomUUID(), accountId, date:get('date'), merchant:get('merchant'), amount:parseMoney(get('amount')), category, note:get('note')};
  });
}
export function importTransactions(ledger: Ledger, incoming: Transaction[]): {ledger:Ledger; added:number} {
  const signature = (t:Transaction) => JSON.stringify([t.accountId,t.date,t.merchant,t.amount,t.note]);
  const seen = new Set(ledger.transactions.map(signature));
  const unique = incoming.filter(t=> { const key=signature(t); if(seen.has(key)) return false; seen.add(key); return true; });
  return {added:unique.length, ledger:{...ledger, transactions:[...unique,...ledger.transactions], accounts:ledger.accounts.map(a=>({...a,balance:a.balance+unique.filter(t=>t.accountId===a.id).reduce((sum,t)=>sum+t.amount,0)}))}};
}
