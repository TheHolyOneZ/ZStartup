import { useState } from 'react';
import { X, FolderOpen } from 'lucide-react';
import { addEntry } from '@/lib/tauri';
import { useUiStore } from '@/store/uiStore';

interface Props {
  onAdded: () => void;
}

export function AddModal({ onAdded }: Props) {
  const closeAddModal = useUiStore((s) => s.closeAddModal);
  const addToast = useUiStore((s) => s.addToast);
  const [name, setName] = useState('');
  const [command, setCommand] = useState('');
  const [scope, setScope] = useState<'current_user' | 'all_users'>('current_user');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) { setError('Name is required'); return; }
    if (!command.trim()) { setError('Command / path is required'); return; }
    setLoading(true); setError('');
    try {
      await addEntry(name.trim(), command.trim(), scope);
      addToast({ type: 'success', message: `"${name}" added to startup` });
      onAdded();
      closeAddModal();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      onClick={(e) => { if (e.target === e.currentTarget) closeAddModal(); }}
      style={{
        position: 'fixed', inset: 0, zIndex: 200,
        background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(4px)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}
    >
      <div style={{
        background: 'var(--surface)', border: '1px solid var(--border)',
        borderRadius: 12, width: 480, padding: 24,
        boxShadow: '0 24px 64px rgba(0,0,0,0.5)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 20 }}>
          <span style={{ fontSize: 15, fontWeight: 700 }}>Add Startup Entry</span>
          <button onClick={closeAddModal} style={{ color: 'var(--text-muted)', padding: 4, borderRadius: 4 }}>
            <X size={16} />
          </button>
        </div>

        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
          <div>
            <label style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: '0.06em', display: 'block', marginBottom: 6 }}>
              Name
            </label>
            <input
              value={name} onChange={(e) => setName(e.target.value)}
              placeholder="My App"
              autoFocus
              style={{
                width: '100%', padding: '8px 12px', borderRadius: 6,
                background: 'var(--surface-input)', border: '1px solid var(--border)',
                color: 'var(--text)', fontSize: 13,
              }}
            />
          </div>

          <div>
            <label style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: '0.06em', display: 'block', marginBottom: 6 }}>
              Command / Path
            </label>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                value={command} onChange={(e) => setCommand(e.target.value)}
                placeholder={'"C:\\Program Files\\MyApp\\app.exe" --arg'}
                style={{
                  flex: 1, padding: '8px 12px', borderRadius: 6,
                  background: 'var(--surface-input)', border: '1px solid var(--border)',
                  color: 'var(--text)', fontSize: 13, fontFamily: 'monospace',
                }}
              />
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>
              Tip: wrap paths with spaces in quotes
            </div>
          </div>

          <div>
            <label style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: '0.06em', display: 'block', marginBottom: 8 }}>
              Scope
            </label>
            <div style={{ display: 'flex', gap: 8 }}>
              {(['current_user', 'all_users'] as const).map((s) => (
                <button
                  key={s} type="button" onClick={() => setScope(s)}
                  style={{
                    flex: 1, padding: '7px 0', borderRadius: 6, fontSize: 12, fontWeight: 500,
                    background: scope === s ? 'var(--accent-sub)' : 'var(--surface-input)',
                    border: `1px solid ${scope === s ? 'var(--accent-border)' : 'var(--border)'}`,
                    color: scope === s ? 'var(--accent)' : 'var(--text-dim)',
                    transition: 'all 100ms',
                  }}
                >
                  {s === 'current_user' ? 'Current User' : 'All Users (System)'}
                </button>
              ))}
            </div>
            {scope === 'all_users' && (
              <div style={{ fontSize: 11, color: 'var(--amber)', marginTop: 6, padding: '6px 10px', background: 'var(--amber-sub)', borderRadius: 6, border: '1px solid var(--amber-border)' }}>
                System entries require administrator privileges to add
              </div>
            )}
          </div>

          {error && (
            <div style={{ fontSize: 12, color: 'var(--red)', padding: '8px 12px', background: 'rgba(239,68,68,0.08)', borderRadius: 6, border: '1px solid rgba(239,68,68,0.2)' }}>
              {error}
            </div>
          )}

          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 4 }}>
            <button type="button" onClick={closeAddModal} style={{ padding: '8px 16px', borderRadius: 6, fontSize: 13, color: 'var(--text-dim)', background: 'var(--surface-input)', border: '1px solid var(--border)' }}>
              Cancel
            </button>
            <button
              type="submit" disabled={loading}
              style={{
                padding: '8px 20px', borderRadius: 6, fontSize: 13, fontWeight: 600,
                background: 'var(--accent)', color: '#fff',
                opacity: loading ? 0.6 : 1, transition: 'opacity 100ms',
              }}
            >
              {loading ? 'Adding...' : 'Add Entry'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
