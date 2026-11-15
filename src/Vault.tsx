import { useState, type FormEvent } from 'react';
import { Fingerprint, LockKeyhole, ShieldCheck } from 'lucide-react';
import { demoLedger } from './demo';
import { migrateLegacyVault, recoverPreviousVault, setupVault, unlockBiometric, unlockVault, type VaultStatus } from './storage';
import './vault.css';

export function VaultGate({status,onUnlocked}:{status:VaultStatus;onUnlocked:()=>Promise<void>}) {
 const [passphrase,setPassphrase]=useState(''); const [confirm,setConfirm]=useState(''); const [source,setSource]=useState(status.legacy_files[0]||'');
 const [error,setError]=useState(''); const [busy,setBusy]=useState(false); const [confirmRecovery,setConfirmRecovery]=useState(false);
 async function submit(event:FormEvent) {
  event.preventDefault();setError('');setBusy(true);
  try {
   if(status.vault_exists) await unlockVault(passphrase);
   else if(status.legacy_files.length) {
    if(passphrase!==confirm) throw new Error('Passphrases do not match.');
    await migrateLegacyVault(passphrase,source);
   } else {
    if(passphrase!==confirm) throw new Error('Passphrases do not match.');
    await setupVault(passphrase,demoLedger());
   }
   setPassphrase('');setConfirm('');await onUnlocked();
  } catch(e) {setError((e as Error).message||'Could not unlock vault.');setPassphrase('');setConfirm('');}
  finally {setBusy(false);}
 }
 async function biometric() {
  setError('');setBusy(true);
  try { await unlockBiometric(); await onUnlocked(); }
  catch(e) {setError((e as Error).message||'Biometric authorization failed. Use your passphrase.');}
  finally {setBusy(false);}
 }
 async function recover() {
  setError('');setBusy(true);
  try { await recoverPreviousVault(passphrase);setPassphrase('');setConfirmRecovery(false);await onUnlocked(); }
  catch(e) {setError((e as Error).message||'Could not recover previous vault.');setPassphrase('');}
  finally {setBusy(false);}
 }
 const initial=!status.vault_exists;
 return <main className="vault-screen"><section className="vault-card">
  <span className="settings-icon mint"><LockKeyhole/></span>
  <p className="eyebrow">GELDAPP · PRIVATE BY DESIGN</p>
  <h1>{initial?(status.legacy_files.length?'Protect your saved data.':'Create your private vault.'):'Welcome back.'}</h1>
  <p>{initial?(status.legacy_files.length?'Choose one existing ledger snapshot to encrypt. The original files stay in place until you review them.':'Your ledger will be encrypted on this device. Keep your passphrase safe; it is the recovery key.'):'Unlock your encrypted ledger with your passphrase.'}</p>
  {status.legacy_files.length>0&&!status.vault_exists&&<div className="vault-migration"><strong>Plaintext snapshots found</strong><label className="field"><span>Choose the version to migrate</span><select value={source} onChange={e=>setSource(e.target.value)}>{status.legacy_files.map(name=><option key={name}>{name}</option>)}</select></label><small>{status.legacy_files.join(' · ')} stay as plaintext until you verify the vault and remove them yourself.</small></div>}
  <form onSubmit={submit}>
   <label className="field"><span>Passphrase</span><input type="password" autoComplete={initial?'new-password':'current-password'} minLength={initial?12:1} required value={passphrase} onChange={e=>setPassphrase(e.target.value)}/></label>
   {initial&&<label className="field"><span>Confirm passphrase</span><input type="password" autoComplete="new-password" minLength={12} required value={confirm} onChange={e=>setConfirm(e.target.value)}/></label>}
   {error&&<p className="form-error" role="alert">{error}</p>}
   <button className="button primary full-width" disabled={busy}>{busy?'Securing…':initial?(status.legacy_files.length?'Encrypt selected snapshot':'Create encrypted vault'):'Unlock vault'} <ShieldCheck size={17}/></button>
  </form>
  {!initial&&status.biometric_enabled&&status.biometric_available&&<button className="button secondary full-width vault-biometric" disabled={busy} onClick={biometric}><Fingerprint size={18}/> Unlock with Touch ID</button>}
  {!initial&&status.biometric_enabled&&!status.biometric_available&&<p className="muted">Biometric unlocking is unavailable right now. Use your passphrase.</p>}
  {!initial&&status.previous_vault_exists&&<div className="vault-recovery"><strong>Previous encrypted snapshot available</strong><small>Recovery rolls back to the last encrypted save. The current file is preserved separately, and the previous copy remains unchanged.</small><label className="checkbox-field"><input type="checkbox" checked={confirmRecovery} onChange={e=>setConfirmRecovery(e.target.checked)}/>I understand this restores the prior snapshot.</label><button type="button" className="button secondary full-width" disabled={busy||!confirmRecovery||!passphrase} onClick={recover}>Recover previous snapshot</button></div>}
  {initial&&status.legacy_files.length===0&&<small className="muted">A small sample ledger is added so you can explore the app. Replace it with your own data when ready.</small>}
 </section></main>;
}
