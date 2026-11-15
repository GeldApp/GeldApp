import { isTauri, invoke } from '@tauri-apps/api/core';
import { validateLedger, type Ledger } from './domain';
const key='geldapp.ledger.v1';
export async function loadLedger():Promise<Ledger|null> {
  const raw = isTauri() ? await invoke<string|null>('load_ledger') : localStorage.getItem(key);
  return raw ? validateLedger(JSON.parse(raw)) : null;
}
export async function saveLedger(ledger:Ledger) {
  const data=JSON.stringify(ledger);
  if(isTauri()) await invoke('save_ledger',{data}); else localStorage.setItem(key,data);
}
export function download(name:string, data:string, type='application/json') {
  const url=URL.createObjectURL(new Blob([data],{type}));
  const a=document.createElement('a'); a.href=url; a.download=name; a.click(); setTimeout(()=>URL.revokeObjectURL(url),1000);
}
function base64(bytes:Uint8Array) { return btoa(Array.from(bytes,c=>String.fromCharCode(c)).join('')); }
function unbase64(value:string) { return Uint8Array.from(atob(value),c=>c.charCodeAt(0)); }
async function derive(password:string,salt:Uint8Array, usages:KeyUsage[]) {
  const material=await crypto.subtle.importKey('raw',new TextEncoder().encode(password),'PBKDF2',false,['deriveKey']);
  return crypto.subtle.deriveKey({name:'PBKDF2',salt,iterations:600000,hash:'SHA-256'},material,{name:'AES-GCM',length:256},false,usages);
}
export async function encryptBackup(ledger:Ledger,password:string) {
  if(password.length<12) throw new Error('Use a passphrase with at least 12 characters.');
  const salt=crypto.getRandomValues(new Uint8Array(16)); const iv=crypto.getRandomValues(new Uint8Array(12));
  const key=await derive(password,salt,['encrypt']);
  const ciphertext=await crypto.subtle.encrypt({name:'AES-GCM',iv},key,new TextEncoder().encode(JSON.stringify(ledger)));
  return JSON.stringify({format:'geldapp-encrypted',version:1,kdf:'PBKDF2-SHA256',iterations:600000,salt:base64(salt),iv:base64(iv),ciphertext:base64(new Uint8Array(ciphertext))});
}
export async function decryptBackup(raw:string,password:string) {
  const data=JSON.parse(raw);
  if(data.format!=='geldapp-encrypted'||data.version!==1||data.kdf!=='PBKDF2-SHA256'||data.iterations!==600000) throw new Error('Unsupported encrypted backup.');
  try {
    const salt=unbase64(data.salt),iv=unbase64(data.iv);
    if(salt.length!==16||iv.length!==12) throw new Error();
    const key=await derive(password,salt,['decrypt']);
    const decrypted=await crypto.subtle.decrypt({name:'AES-GCM',iv},key,unbase64(data.ciphertext));
    return validateLedger(JSON.parse(new TextDecoder().decode(decrypted)));
  } catch { throw new Error('Could not unlock backup. Check your passphrase and file.'); }
}
