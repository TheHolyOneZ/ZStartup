import { create } from 'zustand';

export interface ContextMenuState {
  x: number;
  y: number;
  entry: import('@/lib/tauri').StartupEntry;
}

interface UiStore {
  addModalOpen: boolean;
  contextMenu: ContextMenuState | null;
  toasts: Toast[];
  isAdmin: boolean;
  openAddModal: () => void;
  closeAddModal: () => void;
  setContextMenu: (m: ContextMenuState | null) => void;
  addToast: (t: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;
  setIsAdmin: (v: boolean) => void;
}

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  message: string;
}

let toastId = 0;

export const useUiStore = create<UiStore>((set) => ({
  addModalOpen: false,
  contextMenu: null,
  toasts: [],
  isAdmin: false,
  openAddModal: () => set({ addModalOpen: true }),
  closeAddModal: () => set({ addModalOpen: false }),
  setContextMenu: (contextMenu) => set({ contextMenu }),
  setIsAdmin: (isAdmin) => set({ isAdmin }),
  addToast: (t) => {
    const id = String(++toastId);
    set((s) => ({ toasts: [...s.toasts, { ...t, id }] }));
    setTimeout(() => set((s) => ({ toasts: s.toasts.filter((x) => x.id !== id) })), 3500);
  },
  removeToast: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));
