import { invoke } from '@tauri-apps/api/core';

export type Source = 'registry' | 'folder' | 'packaged';
export type Scope = 'current_user' | 'all_users';
export type EntryType = 'run' | 'run_once';

export interface StartupEntry {
  id: string;
  name: string;
  key: string;
  command: string;
  exe_path: string;
  args: string;
  source: Source;
  source_id: string;
  scope: Scope;
  enabled: boolean;
  file_exists: boolean;
  entry_type: EntryType;
}

export const getStartupEntries = () =>
  invoke<StartupEntry[]>('get_startup_entries');

export const toggleEntry = (entry: StartupEntry, enabled: boolean) =>
  invoke<void>('toggle_entry', { sourceId: entry.source_id, key: entry.key, source: entry.source, enabled });

export const deleteEntry = (entry: StartupEntry) =>
  invoke<void>('delete_entry', { sourceId: entry.source_id, key: entry.key, source: entry.source });

export const sourceBadge = (entry: StartupEntry) => {
  if (entry.source === 'folder') return { label: 'Folder', color: 'var(--teal)', bg: 'var(--teal-sub)' };
  if (entry.source === 'packaged') return { label: 'App', color: 'var(--green)', bg: 'var(--green-sub)' };
  if (entry.entry_type === 'run_once') return { label: 'Once', color: 'var(--accent)', bg: 'var(--accent-sub)' };
  return { label: 'Reg', color: 'var(--blue)', bg: 'var(--blue-sub)' };
};

export const brokenHint = (entry: StartupEntry) =>
  `Target not found: ${entry.exe_path || entry.command}\nThe program was probably uninstalled or moved, so this entry does nothing at startup. It is safe to delete if you no longer use the program.`;

export const addEntry = (name: string, command: string, scope: string) =>
  invoke<void>('add_entry', { name, command, scope });

export const openFileLocation = (path: string) =>
  invoke<void>('open_file_location', { path });

export const isAdmin = () =>
  invoke<boolean>('is_admin');

export const restartAsAdmin = () =>
  invoke<void>('restart_as_admin');
