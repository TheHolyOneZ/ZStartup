import { useState } from 'react';
import { FolderOpen, Trash2, AlertTriangle, Zap } from 'lucide-react';
import type { StartupEntry } from '@/lib/tauri';
import { toggleEntry, openFileLocation } from '@/lib/tauri';
import { useStartupStore } from '@/store/startupStore';
import { useUiStore } from '@/store/uiStore';

interface Props {
  entry: StartupEntry;
  onDelete: (entry: StartupEntry) => void;
  zebra: boolean;
}

const SOURCE_LABELS: Record<string, { label: string; color: string; bg: string }> = {
  reg_user_run:       { label: 'Reg', color: 'var(--blue)', bg: 'var(--blue-sub)' },
  reg_system_run:     { label: 'Reg', color: 'var(--blue)', bg: 'var(--blue-sub)' },
  reg_user_runonce:   { label: 'Once', color: 'var(--accent)', bg: 'var(--accent-sub)' },
  reg_system_runonce: { label: 'Once', color: 'var(--accent)', bg: 'var(--accent-sub)' },
  folder_user:        { label: 'Folder', color: 'var(--teal)', bg: 'var(--teal-sub)' },
  folder_system:      { label: 'Folder', color: 'var(--teal)', bg: 'var(--teal-sub)' },
};

export function EntryRow({ entry, onDelete, zebra }: Props) {
  const updateEntry = useStartupStore((s) => s.updateEntry);
  const addToast = useUiStore((s) => s.addToast);
  const setContextMenu = useUiStore((s) => s.setContextMenu);
  const [toggling, setToggling] = useState(false);
  const [hovered, setHovered] = useState(false);

  const src = SOURCE_LABELS[entry.source_id] ?? { label: 'Reg', color: 'var(--blue)', bg: 'var(--blue-sub)' };
  const isSystem = entry.scope === 'all_users';
  const isRunOnce = entry.entry_type === 'run_once';

  const handleToggle = async () => {
    if (toggling) return;
    setToggling(true);
    const newEnabled = !entry.enabled;
    updateEntry(entry.id, { enabled: newEnabled });
    try {
      await toggleEntry(entry.source_id, entry.name, entry.source, newEnabled);
      addToast({ type: 'success', message: `${entry.name} ${newEnabled ? 'enabled' : 'disabled'}` });
    } catch (e) {
      updateEntry(entry.id, { enabled: entry.enabled });
      addToast({ type: 'error', message: String(e) });
    } finally {
      setToggling(false);
    }
  };

  const shortPath = entry.exe_path.length > 52
    ? '...' + entry.exe_path.slice(-49)
    : entry.exe_path;

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onContextMenu={(e) => { e.preventDefault(); setContextMenu({ x: e.clientX, y: e.clientY, entry }); }}
      style={{
        display: 'grid',
        gridTemplateColumns: '40px 1fr 2fr 80px 72px 100px',
        alignItems: 'center', gap: 8,
        padding: '0 12px', height: 44,
        background: hovered ? 'var(--surface-hover)' : zebra ? 'rgba(255,255,255,0.012)' : 'transparent',
        borderBottom: '1px solid var(--border)',
        borderLeft: !entry.file_exists ? '2px solid var(--amber)' : '2px solid transparent',
        transition: 'background 80ms',
        cursor: 'default',
      }}
    >
      
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <button
          onClick={handleToggle}
          title={entry.enabled ? 'Click to disable' : 'Click to enable'}
          style={{
            width: 28, height: 16, borderRadius: 8,
            background: entry.enabled ? 'var(--green)' : 'var(--surface-input)',
            border: `1px solid ${entry.enabled ? 'var(--green)' : 'var(--border-strong)'}`,
            position: 'relative', transition: 'background 150ms, border-color 150ms',
            opacity: toggling ? 0.5 : 1,
          }}
        >
          <div style={{
            width: 10, height: 10, borderRadius: '50%', background: '#fff',
            position: 'absolute', top: 2,
            left: entry.enabled ? 14 : 2,
            transition: 'left 150ms',
          }} />
        </button>
      </div>

      
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, minWidth: 0 }}>
        {!entry.file_exists && (
          <AlertTriangle size={12} color="var(--amber)" style={{ flexShrink: 0 }} />
        )}
        {isRunOnce && (
          <span title="RunOnce — runs once on next boot then removes itself">
            <Zap size={11} color="var(--accent)" style={{ flexShrink: 0 }} />
          </span>
        )}
        <span style={{
          fontSize: 13, fontWeight: 500,
          color: entry.enabled ? 'var(--text)' : 'var(--text-muted)',
          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {entry.name}
        </span>
      </div>

      
      <div
        title={entry.exe_path}
        style={{
          fontSize: 11, color: 'var(--text-dim)',
          fontFamily: 'monospace', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
          opacity: entry.enabled ? 1 : 0.5,
        }}
      >
        {shortPath || '—'}
      </div>

      
      <div>
        <span style={{
          fontSize: 10, fontWeight: 600, padding: '2px 7px', borderRadius: 'var(--r-full)',
          background: src.bg, color: src.color,
          border: `1px solid ${src.color}40`,
        }}>
          {src.label}
        </span>
      </div>

      
      <div>
        <span style={{
          fontSize: 10, fontWeight: 600, padding: '2px 7px', borderRadius: 'var(--r-full)',
          background: isSystem ? 'var(--amber-sub)' : 'var(--surface-input)',
          color: isSystem ? 'var(--amber)' : 'var(--text-muted)',
          border: `1px solid ${isSystem ? 'var(--amber-border)' : 'var(--border)'}`,
        }}>
          {isSystem ? 'System' : 'User'}
        </span>
      </div>

      
      <div style={{ display: 'flex', alignItems: 'center', gap: 4, justifyContent: 'flex-end', opacity: hovered ? 1 : 0, transition: 'opacity 100ms' }}>
        <button
          onClick={() => openFileLocation(entry.exe_path)}
          title="Open file location"
          onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--text)')}
          onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-muted)')}
          style={{ color: 'var(--text-muted)', padding: 4, borderRadius: 4, transition: 'color 100ms, background 100ms' }}
        >
          <FolderOpen size={14} />
        </button>
        <button
          onClick={() => onDelete(entry)}
          title="Delete"
          onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--red)'; e.currentTarget.style.background = 'var(--red-sub)'; }}
          onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--text-muted)'; e.currentTarget.style.background = 'transparent'; }}
          style={{ color: 'var(--text-muted)', padding: 4, borderRadius: 4, transition: 'color 100ms, background 100ms' }}
        >
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );
}
