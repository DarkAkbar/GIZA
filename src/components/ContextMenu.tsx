import { useEffect, useRef, useState, type ReactNode } from "react";

export type ContextMenuItem = {
  id: string;
  label: string;
  danger?: boolean;
  disabled?: boolean;
  onSelect: () => void;
};

type ContextMenuProps = {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
  title?: string;
};

export function ContextMenu({ x, y, items, onClose, title }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onPointerDown(event: MouseEvent) {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        onClose();
      }
    }
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("mousedown", onPointerDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onPointerDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const maxX = window.innerWidth - rect.width - 8;
    const maxY = window.innerHeight - rect.height - 8;
    el.style.left = `${Math.min(x, maxX)}px`;
    el.style.top = `${Math.min(y, maxY)}px`;
  }, [x, y, items]);

  return (
    <div
      ref={ref}
      className="fixed z-[200] min-w-[12rem] rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-elevated)] py-1 shadow-2xl shadow-black/40"
      style={{ left: x, top: y }}
      role="menu"
    >
      {title ? (
        <p className="border-b border-[var(--color-border)] px-3 py-2 text-[10px] uppercase tracking-wide text-[var(--color-text-muted)]">
          {title}
        </p>
      ) : null}
      <ul>
        {items.map((item) => (
          <li key={item.id}>
            <button
              type="button"
              role="menuitem"
              disabled={item.disabled}
              className={[
                "w-full px-3 py-2 text-left text-sm transition disabled:opacity-40",
                item.danger
                  ? "text-red-300 hover:bg-red-500/15"
                  : "text-[var(--color-text)] hover:bg-[var(--color-surface)]",
              ].join(" ")}
              onClick={() => {
                if (item.disabled) return;
                onClose();
                item.onSelect();
              }}
            >
              {item.label}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

type ContextMenuState = {
  x: number;
  y: number;
  items: ContextMenuItem[];
  title?: string;
};

export function useContextMenu() {
  const [menu, setMenu] = useState<ContextMenuState | null>(null);

  function open(event: React.MouseEvent, items: ContextMenuItem[], title?: string) {
    event.preventDefault();
    event.stopPropagation();
    setMenu({ x: event.clientX, y: event.clientY, items, title });
  }

  function close() {
    setMenu(null);
  }

  const menuPortal: ReactNode = menu ? (
    <ContextMenu
      x={menu.x}
      y={menu.y}
      items={menu.items}
      title={menu.title}
      onClose={close}
    />
  ) : null;

  return { open, close, menuPortal };
}
