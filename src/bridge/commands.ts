import { invoke } from '@tauri-apps/api/core';
import type { CommandMap } from './types';

export const request = <K extends keyof CommandMap>(command: K, args: CommandMap[K]['args']): Promise<CommandMap[K]['result']> =>
  invoke(command, args);
