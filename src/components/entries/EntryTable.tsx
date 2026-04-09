import { useMemo } from 'react';
import { ChevronUp, ChevronDown, ChevronsUpDown } from 'lucide-react';
import { useStartupStore, type SortBy } from '@/store/startupStore';
import { useUiStore } from '@/store/uiStore';
import { deleteEntry } from '@/lib/tauri';
import { EntryRow } from './EntryRow';
import type { StartupEntry } from '@/lib/tauri';

export function EntryTable() {
  const entries   = useStartupStore((s) => s.entries);
  const filter    = useStartupStore((s) => s.filter);
  const search    = useStartupStore((s) => s.search);
  const sortBy    = useStartupStore((s) => s.sortBy);
  const sortAsc   = useStartupStore((s) => s.sortAsc);
  const setSortBy = useStartupStore((s) => s.setSortBy);
  const toggleSort = useStartupStore((s) => s.toggleSort);
  const removeEntry = useStartupStore((s) => s.removeEntry);
  const setPendingDelete = useStartupStore((s) => s.setPendingDelete);
  const setDeleteTimer   = useStartupStore((s) => s.setDeleteTimer);
  const pendingDelete    = useStartupStore((s) => s.pendingDelete);
  const deleteTimer      = useStartupStore((s) => s.deleteTimer);
  const addToast  = useUiStore((s) => s.addToast);

  const filtered = useMemo(() => {
    let list = [...entries];

    if (search) {
      const q = search.toLowerCase();
      list = list.filter((e) =>
        e.name.toLowerCase().includes(q) ||
        e.exe_path.toLowerCase().includes(q) ||
        e.command.toLowerCase().includes(q)
      );
    }

    switch (filter) {
      case 'registry':     list = list.filter((e) => e.source === 'registry'); break;
      case 'folder':       list = list.filter((e) => e.source === 'folder'); break;
      case 'enabled':      list = list.filter((e) => e.enabled); break;
      case 'disabled':     list = list.filter((e) => !e.enabled); break;
      case 'broken':       list = list.filter((e) => !e.file_exists); break;
      case 'current_user': list = list.filter((e) => e.scope === 'current_user'); break;
      case 'all_users':    list = list.filter((e) => e.scope === 'all_users'); break;
    }

    list.sort((a, b) => {
      let cmp = 0;
      if (sortBy === 'name')   cmp = a.name.localeCompare(b.name);
      if (sortBy === 'source') cmp = a.source.localeCompare(b.source);
      if (sortBy === 'status') cmp = Number(b.enabled) - Number(a.enabled);
      if (sortBy === 'scope')  cmp = a.scope.localeCompare(b.scope);
      return sortAsc ? cmp : -cmp;
    });

    return list;
  }, [entries, filter, search, sortBy, sortAsc]);

  const handleSort = (col: SortBy) => {
    if (sortBy === col) toggleSort();
    else setSortBy(col);
  };

  const SortIcon = ({ col }: { col: SortBy }) => {
    if (sortBy !== col) return <ChevronsUpDown size={10} strokeWidth={2} style={{ opacity: 0.3 }} />;
    return sortAsc
      ? <ChevronUp size={11} strokeWidth={2.5} />
      : <ChevronDown size={11} strokeWidth={2.5} />;
  };

  const handleDelete = (entry: StartupEntry) => {
    if (deleteTimer) clearTimeout(deleteTimer);
    removeEntry(entry.id);
    setPendingDelete(entry);
    addToast({ type: 'info', message: `Deleted "${entry.name}" — ` });

    const timer = setTimeout(async () => {
      try {
        await deleteEntry(entry.source_id, entry.name, entry.source, entry.enabled);
      } catch (e) {
        addToast({ type: 'error', message: String(e) });
      }
      setPendingDelete(null);
      setDeleteTimer(null);
    }, 5500);
    setDeleteTimer(timer);
  };

  const headerCell = (label: string, col: SortBy) => (
    <span
      onClick={() => handleSort(col)}
      style={{
        display: 'flex', alignItems: 'center', gap: 4, cursor: 'pointer',
        color: sortBy === col ? 'var(--accent)' : 'var(--text-muted)',
        userSelect: 'none', transition: 'color 100ms',
        fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.06em',
      }}
    >
      {label} <SortIcon col={col} />
    </span>
  );

  return (
    <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column' }}>
      
      <div style={{
        display: 'grid', gridTemplateColumns: '40px 1fr 2fr 80px 72px 100px',
        alignItems: 'center', gap: 8, padding: '0 12px', height: 32, flexShrink: 0,
        background: 'var(--surface)', borderBottom: '1px solid var(--border)',
      }}>
        <div />
        {headerCell('Name', 'name')}
        <span style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.06em', color: 'var(--text-muted)' }}>Command</span>
        {headerCell('Source', 'source')}
        {headerCell('Scope', 'scope')}
        <span style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.06em', color: 'var(--text-muted)', textAlign: 'right' }}>Actions</span>
      </div>

      {filtered.length === 0 ? (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--text-muted)', fontSize: 13 }}>
          No entries match your filter
        </div>
      ) : (
        filtered.map((entry, i) => (
          <EntryRow key={entry.id} entry={entry} onDelete={handleDelete} zebra={i % 2 === 1} />
        ))
      )}
    </div>
  );
}
