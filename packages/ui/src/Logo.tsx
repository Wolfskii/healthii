export function Logo({ title = "Healthii" }: { title?: string }) {
  return (
    <span className="hii-brand">
      <svg className="hii-mark" viewBox="0 0 32 32" aria-hidden="true">
        <rect width="32" height="32" rx="10" fill="currentColor" opacity="0.08" />
        <path
          d="M6 18c3-8 5-8 8 0s5 8 8 0 3-8 4-8"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.2"
          strokeLinecap="round"
        />
      </svg>
      <strong>{title}</strong>
    </span>
  );
}
