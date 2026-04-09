import { useEffect, useState, useCallback } from 'react';
import { Plus, RefreshCw, Search } from 'lucide-react';
import '@/styles/global.css';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getStartupEntries, isAdmin as checkAdmin } from '@/lib/tauri';
import { useStartupStore } from '@/store/startupStore';
import { useUiStore } from '@/store/uiStore';
import { Titlebar } from '@/components/layout/Titlebar';
import { Sidebar } from '@/components/layout/Sidebar';
import { StatusBar } from '@/components/layout/StatusBar';
import { EntryTable } from '@/components/entries/EntryTable';
import { AddModal } from '@/components/entries/AddModal';
import { ToastContainer } from '@/components/ui/Toast';
import { ContextMenu } from '@/components/ui/ContextMenu';
import { AdminBanner } from '@/components/ui/AdminBanner';

export default function App() {
  const setEntries  = useStartupStore((s) => s.setEntries);
  const setLoading  = useStartupStore((s) => s.setLoading);
  const loading     = useStartupStore((s) => s.loading);
  const search      = useStartupStore((s) => s.search);
  const setSearch   = useStartupStore((s) => s.setSearch);
  const addModalOpen   = useUiStore((s) => s.addModalOpen);
  const openAddModal   = useUiStore((s) => s.openAddModal);
  const setIsAdmin     = useUiStore((s) => s.setIsAdmin);
  const [lastRefreshed, setLastRefreshed] = useState<Date | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  
  useEffect(() => {
    const prevent = (e: MouseEvent) => e.preventDefault();
    document.addEventListener('contextmenu', prevent);
    return () => document.removeEventListener('contextmenu', prevent);
  }, []);

  
  useEffect(() => {
    checkAdmin().then(setIsAdmin).catch(() => setIsAdmin(false));
  }, [setIsAdmin]);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getStartupEntries();
      setEntries(data);
      setLastRefreshed(new Date());
    } finally {
      setLoading(false);
    }
  }, [setEntries, setLoading]);

  
  useEffect(() => { load(); }, [load]);

  
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) load();
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [load]);

  const handleRefresh = async () => {
    setRefreshing(true);
    await load();
    setRefreshing(false);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', overflow: 'hidden' }}>
      <style>{`
        @keyframes toastProgress { from { width: 100% } to { width: 0% } }
        @keyframes spin { from { transform: rotate(0deg) } to { transform: rotate(360deg) } }
      `}</style>

      <Titlebar />
      <AdminBanner />

      
      <div style={{
        display: 'flex', alignItems: 'center', gap: 8, padding: '8px 12px',
        background: 'var(--surface)', borderBottom: '1px solid var(--border)', flexShrink: 0,
      }}>
        
        <div style={{
          display: 'flex', alignItems: 'center', gap: 8,
          flex: 1, padding: '6px 12px', borderRadius: 6,
          background: 'var(--surface-input)', border: '1px solid var(--border)',
        }}>
          <Search size={13} color="var(--text-muted)" />
          <input
            value={search} onChange={(e) => setSearch(e.target.value)}
            placeholder="Search entries..."
            style={{ flex: 1, color: 'var(--text)', fontSize: 13, background: 'transparent' }}
          />
        </div>

        
        <button
          onClick={openAddModal}
          style={{
            display: 'flex', alignItems: 'center', gap: 6,
            padding: '6px 14px', borderRadius: 6, fontSize: 13, fontWeight: 600,
            background: 'var(--accent)', color: '#fff',
            transition: 'opacity 100ms',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.opacity = '0.85')}
          onMouseLeave={(e) => (e.currentTarget.style.opacity = '1')}
        >
          <Plus size={14} /> Add
        </button>

        
        <button
          onClick={handleRefresh}
          title="Refresh"
          style={{
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            width: 34, height: 34, borderRadius: 6,
            background: 'var(--surface-input)', border: '1px solid var(--border)',
            color: 'var(--text-dim)', transition: 'background 100ms, color 100ms',
          }}
          onMouseEnter={(e) => { (e.currentTarget.style.background = 'var(--surface-hover)'); (e.currentTarget.style.color = 'var(--text)'); }}
          onMouseLeave={(e) => { (e.currentTarget.style.background = 'var(--surface-input)'); (e.currentTarget.style.color = 'var(--text-dim)'); }}
        >
          <RefreshCw size={13} style={{ animation: refreshing ? 'spin 0.6s linear infinite' : 'none' }} />
        </button>
      </div>

      
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <Sidebar />
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {loading ? (
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--text-muted)', fontSize: 13 }}>
              Loading startup entries...
            </div>
          ) : (
            <EntryTable />
          )}
        </div>
      </div>

      <StatusBar lastRefreshed={lastRefreshed} />

      {addModalOpen && <AddModal onAdded={load} />}
      <ContextMenu />
      <ToastContainer />
    </div>
  );
}
