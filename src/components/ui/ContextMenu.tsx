import { useEffect, useRef } from 'react';
import { ToggleRight, ToggleLeft, FolderOpen, Copy, Trash2 } from 'lucide-react';
import { useUiStore } from '@/store/uiStore';
import { useStartupStore } from '@/store/startupStore';
import { toggleEntry, deleteEntry, openFileLocation } from '@/lib/tauri';

export function ContextMenu() {
  const ctx = useUiStore((s) => s.contextMenu);
  const setContextMenu = useUiStore((s) => s.setContextMenu);
  const addToast = useUiStore((s) => s.addToast);
  const updateEntry = useStartupStore((s) => s.updateEntry);
  const removeEntry = useStartupStore((s) => s.removeEntry);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ctx) return;
    const close = () => setContextMenu(null);
    window.addEventListener('mousedown', close);
    window.addEventListener('keydown', (e) => { if (e.key === 'Escape') close(); });
    return () => window.removeEventListener('mousedown', close);
  }, [ctx, setContextMenu]);

  if (!ctx) return null;

  const { entry } = ctx;

  
  const menuW = 200, menuH = 200;
  const x = Math.min(ctx.x, window.innerWidth - menuW - 8);
  const y = Math.min(ctx.y, window.innerHeight - menuH - 8);

  const handleToggle = async () => {
    setContextMenu(null);
    const newEnabled = !entry.enabled;
    updateEntry(entry.id, { enabled: newEnabled });
    try {
      await toggleEntry(entry.source_id, entry.name, entry.source, newEnabled);
      addToast({ type: 'success', message: `${entry.name} ${newEnabled ? 'enabled' : 'disabled'}` });
    } catch (e) {
      updateEntry(entry.id, { enabled: entry.enabled });
      addToast({ type: 'error', message: String(e) });
    }
  };

  const handleOpenLocation = async () => {
    setContextMenu(null);
    try { await openFileLocation(entry.exe_path); } catch {}
  };

  const handleCopyName = () => {
    setContextMenu(null);
    navigator.clipboard.writeText(entry.name);
    addToast({ type: 'info', message: 'Name copied' });
  };

  const handleCopyPath = () => {
    setContextMenu(null);
    navigator.clipboard.writeText(entry.command);
    addToast({ type: 'info', message: 'Command copied' });
  };

  const handleDelete = async () => {
    setContextMenu(null);
    removeEntry(entry.id);
    addToast({ type: 'info', message: `Deleted "${entry.name}"` });
    setTimeout(async () => {
      try { await deleteEntry(entry.source_id, entry.name, entry.source, entry.enabled); }
      catch (e) { addToast({ type: 'error', message: String(e) }); }
    }, 5500);
  };

  const items: Array<{ icon: React.ReactNode; label: string; action: () => void; danger?: boolean } | 'sep'> = [
    {
      icon: entry.enabled ? <ToggleLeft size={13} /> : <ToggleRight size={13} />,
      label: entry.enabled ? 'Disable' : 'Enable',
      action: handleToggle,
    },
    'sep',
    { icon: <FolderOpen size={13} />, label: 'Open file location', action: handleOpenLocation },
    { icon: <Copy size={13} />, label: 'Copy name', action: handleCopyName },
    { icon: <Copy size={13} />, label: 'Copy command', action: handleCopyPath },
    'sep',
    { icon: <Trash2 size={13} />, label: 'Delete', action: handleDelete, danger: true },
  ];

  return (
    <div
      ref={menuRef}
      onMouseDown={(e) => e.stopPropagation()}
      style={{
        position: 'fixed', top: y, left: x, zIndex: 1000,
        background: 'var(--surface)',
        border: '1px solid var(--border-strong)',
        borderRadius: 8, padding: '4px',
        minWidth: 200,
        boxShadow: '0 12px 40px rgba(0,0,0,0.6)',
      }}
    >
      
      <div style={{
        padding: '6px 10px 8px', fontSize: 11,
        color: 'var(--text-muted)', borderBottom: '1px solid var(--border)',
        marginBottom: 4, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        maxWidth: 192,
      }}>
        {entry.name}
      </div>

      {items.map((item, i) =>
        item === 'sep' ? (
          <div key={i} style={{ height: 1, background: 'var(--border)', margin: '4px 0' }} />
        ) : (
          <button
            key={i}
            onClick={item.action}
            style={{
              width: '100%', display: 'flex', alignItems: 'center', gap: 8,
              padding: '6px 10px', borderRadius: 5, fontSize: 13,
              color: item.danger ? 'var(--red)' : 'var(--text-dim)',
              transition: 'background 80ms, color 80ms',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = item.danger ? 'var(--red-sub)' : 'var(--surface-hover)';
              if (!item.danger) e.currentTarget.style.color = 'var(--text)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'transparent';
              e.currentTarget.style.color = item.danger ? 'var(--red)' : 'var(--text-dim)';
            }}
          >
            <span style={{ opacity: 0.8 }}>{item.icon}</span>
            {item.label}
          </button>
        )
      )}
    </div>
  );
}
