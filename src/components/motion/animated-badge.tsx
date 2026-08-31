// Adapted from the MIT-licensed beUI Animated Badge.
// https://beui.dev/components/motion/animated-badge
import {
  AlertTriangle,
  Check,
  Circle,
  LoaderCircle,
  X,
  type LucideIcon,
} from "lucide-react";
import { AnimatePresence, motion, useReducedMotion, type Variants } from "motion/react";
import type { ReactNode } from "react";
import { EASE_OUT } from "@/lib/ease";
import { cn } from "@/lib/utils";

export type AnimatedBadgeStatus = "neutral" | "success" | "warning" | "danger" | "loading";

const ICONS: Record<AnimatedBadgeStatus, LucideIcon> = {
  neutral: Circle,
  success: Check,
  warning: AlertTriangle,
  danger: X,
  loading: LoaderCircle,
};

const STATUS_CLASS: Record<AnimatedBadgeStatus, string> = {
  neutral: "border-border bg-card text-muted-foreground",
  success: "border-primary/25 bg-primary/8 text-primary",
  warning: "border-amber-400/25 bg-amber-400/8 text-amber-300",
  danger: "border-red-400/25 bg-red-400/8 text-red-300",
  loading: "border-foreground/20 bg-foreground/6 text-foreground",
};

const SPRING_LAYOUT_BADGE = { type: "spring", stiffness: 240, damping: 25, mass: 0.75 } as const;

const ROLL: Variants = {
  initial: { opacity: 0, y: "80%", filter: "blur(5px)" },
  animate: {
    opacity: 1,
    y: 0,
    filter: "blur(0px)",
    transition: { y: SPRING_LAYOUT_BADGE, opacity: { duration: 0.2 }, filter: { duration: 0.3 } },
  },
  exit: { opacity: 0, y: "-80%", filter: "blur(5px)", transition: { duration: 0.16, ease: EASE_OUT } },
};

export function AnimatedBadge({
  status = "neutral",
  children,
  className,
}: {
  status?: AnimatedBadgeStatus;
  children: ReactNode;
  className?: string;
}) {
  const reduce = useReducedMotion();
  const Icon = ICONS[status];
  return (
    <motion.span
      layout
      className={cn(
        "relative inline-flex h-6 shrink-0 items-center gap-1.5 overflow-hidden whitespace-nowrap rounded-full border px-2 text-[10px] font-medium",
        STATUS_CLASS[status],
        className,
      )}
    >
      {status === "loading" && !reduce ? (
        <motion.span
          aria-hidden
          className="absolute inset-0 rounded-full bg-current opacity-[.06]"
          animate={{ scale: [0.94, 1.08, 0.94], opacity: [0.04, 0.1, 0.04] }}
          transition={{ duration: 1.6, repeat: Infinity }}
        />
      ) : null}
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.span
          key={`${status}-icon`}
          variants={reduce ? undefined : ROLL}
          initial={reduce ? false : "initial"}
          animate={reduce ? { opacity: 1 } : "animate"}
          exit={reduce ? undefined : "exit"}
          className="relative z-10 inline-flex"
        >
          {status === "loading" && !reduce ? (
            <motion.span animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: "linear" }}>
              <Icon className="size-3" />
            </motion.span>
          ) : (
            <Icon className="size-3" />
          )}
        </motion.span>
      </AnimatePresence>
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.span
          key={String(children)}
          variants={reduce ? undefined : ROLL}
          initial={reduce ? false : "initial"}
          animate={reduce ? { opacity: 1 } : "animate"}
          exit={reduce ? undefined : "exit"}
          className="relative z-10"
        >
          {children}
        </motion.span>
      </AnimatePresence>
    </motion.span>
  );
}
