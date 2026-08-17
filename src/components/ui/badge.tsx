import type { HTMLAttributes } from "react";
import { cn } from "../../lib/cn";

export function Badge({ tone = "neutral", className, ...props }: HTMLAttributes<HTMLSpanElement> & { tone?: "neutral" | "positive" | "warning" }) {
  return <span className={cn("badge", `badge-${tone}`, className)} {...props} />;
}
