import type { ButtonHTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { LoaderCircle } from "lucide-react";
import { cn } from "../../lib/cn";

const variants = cva("button", {
  variants: {
    variant: { primary: "button-primary", secondary: "button-secondary", ghost: "button-ghost" },
    size: { sm: "button-sm", md: "button-md", lg: "button-lg" },
  },
  defaultVariants: { variant: "primary", size: "md" },
});

type Props = ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof variants> & { loading?: boolean };

export function Button({ className, variant, size, loading, children, disabled, ...props }: Props) {
  return (
    <button className={cn(variants({ variant, size }), className)} disabled={disabled || loading} {...props}>
      {loading && <LoaderCircle className="spin" aria-hidden="true" />}
      {children}
    </button>
  );
}
