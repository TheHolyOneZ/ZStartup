import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-react';
import { useUiStore, type Toast } from '@/store/uiStore';

const CONFIG = {
  success: { icon: CheckCircle,    color: 'var(--green)', bg: 'rgba(34,197,94,0.1)',   border: 'rgba(34,197,94,0.2)' },
  error:   { icon: AlertCircle,    color: 'var(--red)',   bg: 'rgba(239,68,68,0.1)',   border: 'rgba(239,68,68,0.2)' },
  info:    { icon: Info,           color: 'var(--accent)',bg: 'var(--accent-sub)',      border: 'var(--accent-border)' },
  warning: { icon: AlertTriangle,  color: 'var(--amber)', bg: 'var(--amber-sub)',       border: 'var(--amber-border)' },
};

function ToastItem({ toast }: { toast: Toast }) {
  const removeToast = useUiStore((s) => s.removeToast);
  const cfg = CONFIG[toast.type];
  const Icon = cfg.icon;

  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 10,
      padding: '10px 14px', borderRadius: 8, minWidth: 280, maxWidth: 360,
      background: 'var(--surface)', border: `1px solid ${cfg.border}`,
      boxShadow: '0 8px 24px rgba(0,0,0,0.4)',
      position: 'relative', overflow: 'hidden',
    }}>
      <div style={{
        width: 28, height: 28, borderRadius: 6, flexShrink: 0,
        background: cfg.bg, display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <Icon size={14} color={cfg.color} />
      </div>
      <span style={{ flex: 1, fontSize: 13, color: 'var(--text)', lineHeight: 1.4 }}>{toast.message}</span>
      <button onClick={() => removeToast(toast.id)} style={{ color: 'var(--text-muted)', flexShrink: 0 }}>
        <X size={13} />
      </button>
      
      <div style={{
        position: 'absolute', bottom: 0, left: 0, height: 2,
        background: cfg.color, borderRadius: '0 0 8px 8px',
        animation: 'toastProgress 3.5s linear forwards',
      }} />
    </div>
  );
}

export function ToastContainer() {
  const toasts = useUiStore((s) => s.toasts);
  return (
    <div style={{
      position: 'fixed', bottom: 32, right: 16, zIndex: 999,
      display: 'flex', flexDirection: 'column', gap: 8, alignItems: 'flex-end',
    }}>
      {toasts.map((t) => <ToastItem key={t.id} toast={t} />)}
    </div>
  );
}
