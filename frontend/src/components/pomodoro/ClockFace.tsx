import type { ReactNode } from "react";
import { formatClock } from "../../lib/pomodoro";

interface ClockFaceProps {
  /** What the clock counts: the phase of the cycle, already translated. */
  label: string;
  seconds: number;
  children?: ReactNode;
}

/** The big countdown with its phase label. Extra content (subject, buttons) goes below. */
export default function ClockFace({
  label,
  seconds,
  children,
}: ClockFaceProps) {
  return (
    <div className="flex flex-col items-center gap-3 rounded-xl border border-border p-6 md:p-10">
      <p className="text-sm font-medium text-muted-foreground">{label}</p>
      <p
        role="timer"
        className="text-6xl font-semibold tracking-tight tabular-nums md:text-7xl"
      >
        {formatClock(seconds)}
      </p>
      {children}
    </div>
  );
}
