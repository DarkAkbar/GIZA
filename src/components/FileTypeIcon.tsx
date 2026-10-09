type FileTypeIconProps = {
  extension?: string | null;
  className?: string;
};

const EXT_STYLES: Record<string, { label: string; bg: string; fg: string }> = {
  pdf: { label: "PDF", bg: "bg-red-500/20", fg: "text-red-300" },
  doc: { label: "DOC", bg: "bg-blue-500/20", fg: "text-blue-300" },
  docx: { label: "DOC", bg: "bg-blue-500/20", fg: "text-blue-300" },
  xls: { label: "XLS", bg: "bg-emerald-500/20", fg: "text-emerald-300" },
  xlsx: { label: "XLS", bg: "bg-emerald-500/20", fg: "text-emerald-300" },
  ppt: { label: "PPT", bg: "bg-orange-500/20", fg: "text-orange-300" },
  pptx: { label: "PPT", bg: "bg-orange-500/20", fg: "text-orange-300" },
  zip: { label: "ZIP", bg: "bg-amber-500/20", fg: "text-amber-300" },
  rar: { label: "RAR", bg: "bg-amber-500/20", fg: "text-amber-300" },
  mp4: { label: "VID", bg: "bg-violet-500/20", fg: "text-violet-300" },
  mkv: { label: "VID", bg: "bg-violet-500/20", fg: "text-violet-300" },
  jpg: { label: "IMG", bg: "bg-pink-500/20", fg: "text-pink-300" },
  jpeg: { label: "IMG", bg: "bg-pink-500/20", fg: "text-pink-300" },
  png: { label: "IMG", bg: "bg-pink-500/20", fg: "text-pink-300" },
  exe: { label: "APP", bg: "bg-slate-500/20", fg: "text-slate-300" },
  iso: { label: "ISO", bg: "bg-cyan-500/20", fg: "text-cyan-300" },
};

export function FileTypeIcon({ extension, className = "" }: FileTypeIconProps) {
  const ext = (extension ?? "").toLowerCase().replace(/^\./, "");
  const style = EXT_STYLES[ext] ?? {
    label: ext.slice(0, 3).toUpperCase() || "FIC",
    bg: "bg-[var(--color-border)]/40",
    fg: "text-[var(--color-text-muted)]",
  };

  return (
    <span
      className={`inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-[var(--color-border)] text-[10px] font-bold tracking-tight ${style.bg} ${style.fg} ${className}`}
      title={ext || "fichier"}
    >
      {style.label}
    </span>
  );
}
