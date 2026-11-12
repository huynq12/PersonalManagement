interface ColorDotProps {
  /** Hex color, `#RRGGBB`. */
  color: string;
}

/** A small circle in a subject's color. Decorative: the subject's name is always shown beside it. */
export default function ColorDot({ color }: ColorDotProps) {
  return (
    <span
      aria-hidden="true"
      style={{ backgroundColor: color }}
      className="size-4 shrink-0 rounded-full"
    />
  );
}
