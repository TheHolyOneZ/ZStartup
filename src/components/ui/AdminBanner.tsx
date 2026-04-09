import { ShieldAlert, ShieldCheck } from 'lucide-react';
import { restartAsAdmin } from '@/lib/tauri';
import { useUiStore } from '@/store/uiStore';
import { useState } from 'react';

export function AdminBanner() {
  const isAdmin = useUiStore((s) => s.isAdmin);
  const [restarting, setRestarting] = useState(false);

  if (isAdmin) return null;

  const handleRestart = async () => {
    setRestarting(true);
    try {
      await restartAsAdmin();
    } catch {
      setRestarting(false);
    }
  };

  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 10,
      padding: '7px 14px',
      background: 'rgba(245,158,11,0.08)',
      borderBottom: '1px solid var(--amber-border)',
      flexShrink: 0,
    }}>
      <ShieldAlert size={14} color="var(--amber)" />
      <span style={{ fontSize: 12, color: 'var(--amber)', flex: 1 }}>
        Running without administrator privileges — system entries and some user entries may be read-only
      </span>
      <button
        onClick={handleRestart}
        disabled={restarting}
        style={{
          display: 'flex', alignItems: 'center', gap: 6,
          padding: '4px 12px', borderRadius: 5, fontSize: 12, fontWeight: 600,
          background: 'var(--amber-sub)', color: 'var(--amber)',
          border: '1px solid var(--amber-border)',
          opacity: restarting ? 0.6 : 1, transition: 'opacity 100ms',
          whiteSpace: 'nowrap',
        }}
      >
        <ShieldCheck size={12} />
        {restarting ? 'Restarting...' : 'Restart as Admin'}
      </button>
    </div>
  );
}
