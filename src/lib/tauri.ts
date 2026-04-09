import { invoke } from '@tauri-apps/api/core';

export type Source = 'registry' | 'folder';
export type Scope = 'current_user' | 'all_users';
export type EntryType = 'run' | 'run_once';

export interface StartupEntry {
  id: string;
  name: string;
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

export const toggleEntry = (source_id: string, name: string, source: Source, enabled: boolean) =>
  invoke<void>('toggle_entry', { sourceId: source_id, name, source, enabled });

export const deleteEntry = (source_id: string, name: string, source: Source, enabled: boolean) =>
  invoke<void>('delete_entry', { sourceId: source_id, name, source, enabled });

export const addEntry = (name: string, command: string, scope: string) =>
  invoke<void>('add_entry', { name, command, scope });

export const openFileLocation = (path: string) =>
  invoke<void>('open_file_location', { path });

export const isAdmin = () =>
  invoke<boolean>('is_admin');

export const restartAsAdmin = () =>
  invoke<void>('restart_as_admin');
