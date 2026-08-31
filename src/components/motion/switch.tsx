// Adapted from the MIT-licensed beUI Motion switch component.
// https://beui.dev/components/motion/switch
import { animate, motion, MotionConfig, useReducedMotion } from "motion/react";
import { useEffect, useId, useRef, useState } from "react";
import { cn } from "@/lib/utils";

const THUMB_SPRING = { type: "spring", stiffness: 800, damping: 80, mass: 4 } as const;

export interface SwitchProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
  ariaLabel?: string;
  className?: string;
}

export function Switch({
  checked,
  onCheckedChange,
  disabled,
  label,
  ariaLabel,
  className,
}: SwitchProps) {
  const id = useId();
  const thumbRef = useRef<HTMLDivElement>(null);
  const reduce = useReducedMotion();
  const [isPressed, setIsPressed] = useState(false);

  useEffect(() => {
    if (!thumbRef.current || reduce || !disabled || !isPressed) return;
    void animate(thumbRef.current, { x: [0, -2, 2, -1, 0] }, { duration: 0.45 });
  }, [disabled, isPressed, reduce]);

  return (
    <MotionConfig transition={reduce ? { duration: 0 } : THUMB_SPRING}>
      <span className={cn("inline-flex items-center gap-2.5", className)}>
        <motion.button
          id={id}
          type="button"
          role="switch"
          aria-checked={checked}
          aria-label={ariaLabel}
          disabled={disabled}
          onClick={() => !disabled && onCheckedChange(!checked)}
          onPointerDown={() => setIsPressed(true)}
          onPointerUp={() => setIsPressed(false)}
          onPointerLeave={() => setIsPressed(false)}
          initial={false}
          className={cn(
            "inline-flex h-6 w-10 shrink-0 items-center rounded-full border px-1 outline-none transition-colors duration-200",
            "focus-visible:ring-2 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-60",
            checked
              ? "justify-end border-primary bg-primary"
              : "justify-start border-border-strong bg-muted",
          )}
        >
          <motion.div
            ref={thumbRef}
            layout
            className={cn(
              "pointer-events-none size-4 rounded-full shadow-sm",
              checked ? "bg-primary-foreground" : "bg-muted-foreground",
            )}
          />
        </motion.button>
        {label ? (
          <label htmlFor={id} className="cursor-pointer text-xs text-muted-foreground">
            {label}
          </label>
        ) : null}
      </span>
    </MotionConfig>
  );
}
