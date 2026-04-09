import { create } from 'zustand';
import type { StartupEntry } from '@/lib/tauri';

export type FilterType = 'all' | 'registry' | 'folder' | 'enabled' | 'disabled' | 'broken' | 'current_user' | 'all_users';
export type SortBy = 'name' | 'source' | 'status' | 'scope';

interface StartupStore {
  entries: StartupEntry[];
  loading: boolean;
  search: string;
  filter: FilterType;
  sortBy: SortBy;
  sortAsc: boolean;
  pendingDelete: StartupEntry | null;
  deleteTimer: ReturnType<typeof setTimeout> | null;

  setEntries: (e: StartupEntry[]) => void;
  setLoading: (v: boolean) => void;
  setSearch: (v: string) => void;
  setFilter: (v: FilterType) => void;
  setSortBy: (v: SortBy) => void;
  toggleSort: () => void;
  setPendingDelete: (e: StartupEntry | null) => void;
  setDeleteTimer: (t: ReturnType<typeof setTimeout> | null) => void;
  updateEntry: (id: string, patch: Partial<StartupEntry>) => void;
  removeEntry: (id: string) => void;
}

export const useStartupStore = create<StartupStore>((set) => ({
  entries: [],
  loading: true,
  search: '',
  filter: 'all',
  sortBy: 'name',
  sortAsc: true,
  pendingDelete: null,
  deleteTimer: null,

  setEntries: (entries) => set({ entries }),
  setLoading: (loading) => set({ loading }),
  setSearch: (search) => set({ search }),
  setFilter: (filter) => set({ filter }),
  setSortBy: (sortBy) => set({ sortBy }),
  toggleSort: () => set((s) => ({ sortAsc: !s.sortAsc })),
  setPendingDelete: (pendingDelete) => set({ pendingDelete }),
  setDeleteTimer: (deleteTimer) => set({ deleteTimer }),
  updateEntry: (id, patch) =>
    set((s) => ({ entries: s.entries.map((e) => (e.id === id ? { ...e, ...patch } : e)) })),
  removeEntry: (id) =>
    set((s) => ({ entries: s.entries.filter((e) => e.id !== id) })),
}));
