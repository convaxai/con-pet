// Adapted from the MIT-licensed beUI Motion Button.
// https://beui.dev/components/motion/button
import {
  AnimatePresence,
  motion,
  useReducedMotion,
  type HTMLMotionProps,
} from "motion/react";
import {
  forwardRef,
  useCallback,
  useRef,
  useState,
  type PointerEvent,
  type ReactNode,
} from "react";
import { EASE_OUT, SPRING_PRESS } from "@/lib/ease";
import { useHoverCapable } from "@/lib/use-hover-capable";
import { cn } from "@/lib/utils";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "outline";
export type ButtonSize = "sm" | "md" | "lg" | "icon";

export interface ButtonProps extends Omit<HTMLMotionProps<"button">, "children"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  compact?: boolean;
  pressScale?: number;
  ripple?: boolean;
  children?: ReactNode;
}

type Ripple = { id: number; x: number; y: number; size: number };

const VARIANT_CLASS: Record<ButtonVariant, string> = {
  primary: "border border-primary bg-primary text-primary-foreground hover:bg-primary/88",
  secondary: "border border-border bg-card text-foreground hover:border-border-strong hover:bg-muted",
  ghost: "border border-transparent text-muted-foreground hover:bg-foreground/5 hover:text-foreground",
  outline: "border border-border bg-transparent text-foreground hover:border-border-strong hover:bg-foreground/5",
};

const SIZE_CLASS: Record<ButtonSize, string> = {
  sm: "h-8 gap-1.5 rounded-full px-3 text-xs",
  md: "h-10 gap-2 rounded-full px-5 text-sm",
  lg: "h-12 gap-2 rounded-full px-6 text-base",
  icon: "size-9 rounded-full",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    variant = "ghost",
    size: sizeProp = "md",
    compact = false,
    pressScale = 0.94,
    ripple = false,
    className,
    children,
    onPointerDown,
    type = "button",
    ...rest
  },
  ref,
) {
  const reduce = useReducedMotion();
  const canHover = useHoverCapable();
  const [ripples, setRipples] = useState<Ripple[]>([]);
  const nextId = useRef(0);
  const size = compact ? "sm" : sizeProp;

  const handlePointerDown = useCallback(
    (event: PointerEvent<HTMLButtonElement>) => {
      if (ripple && !reduce) {
        const rect = event.currentTarget.getBoundingClientRect();
        const rippleSize = Math.max(rect.width, rect.height) * 2;
        setRipples((current) => [
          ...current,
          {
            id: nextId.current++,
            x: event.clientX - rect.left,
            y: event.clientY - rect.top,
            size: rippleSize,
          },
        ]);
      }
      onPointerDown?.(event);
    },
    [onPointerDown, reduce, ripple],
  );

  return (
    <motion.button
      ref={ref}
      type={type}
      whileTap={reduce || rest.disabled ? undefined : { scale: pressScale }}
      whileHover={reduce || !canHover || rest.disabled ? undefined : { scale: 1.018 }}
      transition={SPRING_PRESS}
      onPointerDown={handlePointerDown}
      className={cn(
        "inline-flex select-none items-center justify-center font-medium outline-none transition-colors",
        "focus-visible:ring-2 focus-visible:ring-ring/40 disabled:pointer-events-none disabled:opacity-45",
        ripple && "relative overflow-hidden",
        VARIANT_CLASS[variant],
        SIZE_CLASS[size],
        className,
      )}
      {...rest}
    >
      {ripple && !reduce ? (
        <span className="pointer-events-none absolute inset-0 overflow-hidden rounded-[inherit]">
          <AnimatePresence>
            {ripples.map((item) => (
              <motion.span
                key={item.id}
                className="absolute rounded-full bg-current"
                style={{
                  left: item.x,
                  top: item.y,
                  width: item.size,
                  height: item.size,
                  x: "-50%",
                  y: "-50%",
                }}
                initial={{ scale: 0.05, opacity: 0.22 }}
                animate={{ scale: 1, opacity: 0 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 1.25, ease: EASE_OUT }}
                onAnimationComplete={() =>
                  setRipples((current) => current.filter((rippleItem) => rippleItem.id !== item.id))
                }
              />
            ))}
          </AnimatePresence>
        </span>
      ) : null}
      {children}
    </motion.button>
  );
});
