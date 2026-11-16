import { describe, expect, it, vi } from 'vitest';
import { lockWithStatus } from './session';

describe('lockWithStatus', () => {
  it('clears the unlocked view immediately after native lock even if status refresh fails', async () => {
    const events: string[] = [];
    const result = await lockWithStatus(
      Promise.resolve().then(() => events.push('drained')),
      async () => { events.push('saved'); },
      async () => { events.push('locked'); },
      () => { events.push('cleared'); },
      async () => { events.push('status'); throw new Error('status unavailable'); },
    );

    expect(result).toBeNull();
    expect(events).toEqual(['drained', 'saved', 'locked', 'cleared', 'status']);
  });

  it('keeps the workspace open when the final save fails', async () => {
    const clear = vi.fn();
    const lock = vi.fn();
    await expect(lockWithStatus(
      Promise.resolve(),
      async () => { throw new Error('disk full'); },
      lock,
      clear,
      async () => ({ unlocked: false }),
    )).rejects.toThrow('disk full');
    expect(lock).not.toHaveBeenCalled();
    expect(clear).not.toHaveBeenCalled();
  });
});
