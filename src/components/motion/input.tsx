// Adapted from the MIT-licensed beUI Motion input component.
// https://beui.dev/components/motion/input
import { animate, useReducedMotion } from "motion/react";
import {
  forwardRef,
  useEffect,
  useRef,
  type InputHTMLAttributes,
  type ReactNode,
} from "react";
import { cn } from "@/lib/utils";

export interface InputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "onChange"> {
  onValueChange?: (value: string) => void;
  error?: string | boolean;
  leftIcon?: ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { className, onValueChange, error, leftIcon, ...props },
  ref,
) {
  const fieldRef = useRef<HTMLDivElement>(null);
  const reduce = useReducedMotion();

  useEffect(() => {
    if (!fieldRef.current || reduce || !error) return;
    void animate(fieldRef.current, { x: [0, -5, 5, -3, 3, 0] }, { duration: 0.4 });
  }, [error, reduce]);

  return (
    <div
      ref={fieldRef}
      className={cn(
        "relative h-10 overflow-hidden rounded-full border border-border bg-background transition-colors",
        "focus-within:border-foreground/45 focus-within:ring-2 focus-within:ring-ring/25",
        error && "border-red-400 ring-2 ring-red-400/15",
        props.disabled && "opacity-60",
      )}
    >
      {leftIcon ? (
        <span className="pointer-events-none absolute left-3.5 top-1/2 flex -translate-y-1/2 items-center text-muted-foreground [&_svg]:size-4">
          {leftIcon}
        </span>
      ) : null}
      <input
        ref={ref}
        className={cn(
          "h-full w-full rounded-full bg-transparent pr-3 text-sm text-foreground outline-none",
          leftIcon ? "pl-10" : "pl-3",
          "placeholder:text-muted-foreground/60 disabled:cursor-not-allowed",
          className,
        )}
        aria-invalid={Boolean(error) || undefined}
        onChange={(event) => onValueChange?.(event.target.value)}
        {...props}
      />
    </div>
  );
});
