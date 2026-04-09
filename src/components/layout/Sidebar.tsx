import { Database, FolderOpen, ToggleLeft, ToggleRight, Users, User, AlertTriangle, List } from 'lucide-react';
import { useStartupStore, type FilterType } from '@/store/startupStore';
import { useMemo } from 'react';

const filters: { id: FilterType; label: string; icon: React.ReactNode }[] = [
  { id: 'all',          label: 'All',          icon: <List size={13} /> },
  { id: 'registry',     label: 'Registry',     icon: <Database size={13} /> },
  { id: 'folder',       label: 'Folders',      icon: <FolderOpen size={13} /> },
  { id: 'enabled',      label: 'Enabled',      icon: <ToggleRight size={13} /> },
  { id: 'disabled',     label: 'Disabled',     icon: <ToggleLeft size={13} /> },
  { id: 'broken',       label: 'Broken',       icon: <AlertTriangle size={13} /> },
  { id: 'current_user', label: 'User',         icon: <User size={13} /> },
  { id: 'all_users',    label: 'System',       icon: <Users size={13} /> },
];

export function Sidebar() {
  const entries = useStartupStore((s) => s.entries);
  const filter = useStartupStore((s) => s.filter);
  const setFilter = useStartupStore((s) => s.setFilter);

  const counts = useMemo(() => ({
    all:          entries.length,
    registry:     entries.filter((e) => e.source === 'registry').length,
    folder:       entries.filter((e) => e.source === 'folder').length,
    enabled:      entries.filter((e) => e.enabled).length,
    disabled:     entries.filter((e) => !e.enabled).length,
    broken:       entries.filter((e) => !e.file_exists).length,
    current_user: entries.filter((e) => e.scope === 'current_user').length,
    all_users:    entries.filter((e) => e.scope === 'all_users').length,
  }), [entries]);

  return (
    <div style={{
      width: 190, flexShrink: 0,
      background: 'var(--surface)', borderRight: '1px solid var(--border)',
      display: 'flex', flexDirection: 'column',
      overflowY: 'auto',
    }}>
      <div style={{ padding: '12px 10px 6px', fontSize: 10, fontWeight: 600, letterSpacing: '0.08em', textTransform: 'uppercase', color: 'var(--text-muted)' }}>
        Filter
      </div>
      {filters.map((f, i) => {
        const active = filter === f.id;
        const count = counts[f.id] ?? 0;
        const isBroken = f.id === 'broken';
        return (
          <div key={f.id}>
            {(i === 3 || i === 5 || i === 6) && (
              <div style={{ height: 1, background: 'var(--border)', margin: '6px 10px' }} />
            )}
            <button
              onClick={() => setFilter(f.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 8,
                padding: '6px 12px', borderRadius: 6, margin: '1px 6px', width: 'calc(100% - 12px)',
                background: active ? 'var(--accent-sub)' : 'transparent',
                color: active ? 'var(--accent)' : isBroken && count > 0 ? 'var(--amber)' : 'var(--text-dim)',
                fontSize: 13, fontWeight: active ? 600 : 400,
                transition: 'background 100ms, color 100ms',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) => { if (!active) (e.currentTarget as HTMLElement).style.background = 'var(--surface-hover)'; }}
              onMouseLeave={(e) => { if (!active) (e.currentTarget as HTMLElement).style.background = 'transparent'; }}
            >
              <span style={{ opacity: 0.75 }}>{f.icon}</span>
              <span style={{ flex: 1, textAlign: 'left' }}>{f.label}</span>
              <span style={{
                fontSize: 10, fontWeight: 500,
                color: active ? 'var(--accent)' : isBroken && count > 0 ? 'var(--amber)' : 'var(--text-muted)',
                minWidth: 16, textAlign: 'right',
              }}>
                {count}
              </span>
            </button>
          </div>
        );
      })}
    </div>
  );
}
