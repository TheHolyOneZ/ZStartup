import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, Shield } from 'lucide-react';

export function Titlebar() {
  const win = getCurrentWindow();
  return (
    <div
      style={{
        height: 48, display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        background: 'var(--surface)', borderBottom: '1px solid var(--border)',
        flexShrink: 0,
      }}
    >
      <div data-tauri-drag-region style={{ display: 'flex', alignItems: 'center', gap: 8, paddingLeft: 14, flex: 1, height: '100%' }}>
        <div style={{
          width: 26, height: 26, borderRadius: 6, background: 'var(--accent-sub)',
          border: '1px solid var(--accent-border)', display: 'flex', alignItems: 'center', justifyContent: 'center',
        }}>
          <Shield size={13} color="var(--accent)" strokeWidth={2} />
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 6 }}>
            <span style={{ fontSize: 13, fontWeight: 700, letterSpacing: '-0.01em', color: 'var(--text)' }}>
              ZStartup
            </span>
            <span style={{ fontSize: 11, color: 'var(--text-muted)' }}>Startup Manager</span>
          </div>
          <span style={{ fontSize: 10, color: 'var(--text-dim)', letterSpacing: '0.01em' }}>
            by TheHolyOneZ
          </span>
        </div>
      </div>

      <div style={{ display: 'flex' }}>
        {[
          { icon: <Minus size={14} />, action: () => win.minimize(), hover: 'var(--surface-hover)' },
          { icon: <Square size={12} />, action: () => win.toggleMaximize(), hover: 'var(--surface-hover)' },
          { icon: <X size={14} />, action: () => win.close(), hover: 'var(--red-sub)', hoverColor: 'var(--red)' },
        ].map((btn, i) => (
          <button
            key={i}
            onClick={btn.action}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLElement).style.background = btn.hover;
              if (btn.hoverColor) (e.currentTarget as HTMLElement).style.color = btn.hoverColor;
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLElement).style.background = 'transparent';
              (e.currentTarget as HTMLElement).style.color = 'var(--text-dim)';
            }}
            style={{
              width: 46, height: 48, display: 'flex', alignItems: 'center', justifyContent: 'center',
              color: 'var(--text-dim)', transition: 'background 100ms, color 100ms',
            }}
          >
            {btn.icon}
          </button>
        ))}
      </div>
    </div>
  );
}
