import { useStartupStore } from '@/store/startupStore';
import { useMemo } from 'react';

export function StatusBar({ lastRefreshed }: { lastRefreshed: Date | null }) {
  const entries = useStartupStore((s) => s.entries);

  const stats = useMemo(() => ({
    total:    entries.length,
    disabled: entries.filter((e) => !e.enabled).length,
    broken:   entries.filter((e) => !e.file_exists).length,
  }), [entries]);

  const timeStr = lastRefreshed
    ? lastRefreshed.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
    : '—';

  return (
    <div style={{
      height: 26, display: 'flex', alignItems: 'center', gap: 16,
      padding: '0 14px', flexShrink: 0,
      background: 'var(--surface)', borderTop: '1px solid var(--border)',
      fontSize: 11, color: 'var(--text-muted)',
    }}>
      <span>{stats.total} entries</span>
      {stats.disabled > 0 && <span style={{ color: 'var(--text-muted)' }}>· {stats.disabled} disabled</span>}
      {stats.broken > 0 && <span style={{ color: 'var(--amber)' }}>· {stats.broken} broken</span>}
      <span style={{ marginLeft: 'auto' }}>Refreshed {timeStr}</span>
    </div>
  );
}
