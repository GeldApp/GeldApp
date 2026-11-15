export async function lockWithStatus<T>(
  pendingSaves: Promise<unknown>,
  saveLatest: () => Promise<void>,
  lock: () => Promise<void>,
  onLocked: () => void,
  refreshStatus: () => Promise<T>,
): Promise<T | null> {
  await pendingSaves;
  await saveLatest();
  await lock();
  onLocked();
  try {
    return await refreshStatus();
  } catch {
    return null;
  }
}
