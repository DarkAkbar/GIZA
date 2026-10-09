type PageHeaderProps = {
  title: string;
  description: string;
};

export function PageHeader({ title, description }: PageHeaderProps) {
  return (
    <header className="mb-8">
      <h2 className="text-2xl font-semibold tracking-tight text-[var(--color-text)] md:text-[1.65rem]">
        {title}
      </h2>
      <p className="mt-1 max-w-2xl text-sm text-[var(--color-text-muted)]">{description}</p>
    </header>
  );
}
