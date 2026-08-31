// Adapted from the MIT-licensed beUI Bouncy Accordion.
// https://beui.dev/components/motion/bouncy-accordion
import { ChevronDown } from "lucide-react";
import { motion, useReducedMotion, type Transition } from "motion/react";
import { useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { EASE_OUT } from "@/lib/ease";
import { cn } from "@/lib/utils";

const SURFACE_SPRING: Transition = { type: "spring", duration: 0.55, bounce: 0.3 };
const CONTENT_SPRING: Transition = { type: "spring", duration: 0.58, bounce: 0.28 };

export function BouncyAccordion({
  title,
  hint,
  icon,
  open,
  onOpenChange,
  children,
  className,
}: {
  title: ReactNode;
  hint?: ReactNode;
  icon?: ReactNode;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
  className?: string;
}) {
  const reduce = useReducedMotion();
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState(0);

  useLayoutEffect(() => {
    const node = contentRef.current;
    if (!node) return;
    const measure = () => setContentHeight(node.offsetHeight);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  return (
    <motion.section
      layout="position"
      initial={false}
      animate={{ borderRadius: open ? 26 : 18 }}
      transition={reduce ? { duration: 0 } : SURFACE_SPRING}
      className={cn("overflow-visible border border-border bg-card", className)}
    >
      <button
        type="button"
        aria-expanded={open}
        onClick={() => onOpenChange(!open)}
        className="flex min-h-14 w-full items-center gap-3 px-5 text-left outline-none transition-colors hover:bg-foreground/[.025] focus-visible:bg-foreground/[.04]"
      >
        {icon ? (
          <span className="grid size-8 shrink-0 place-items-center rounded-full border border-border bg-background text-muted-foreground">
            {icon}
          </span>
        ) : null}
        <span className="min-w-0 flex-1">
          <strong className="block text-[12px] font-medium text-foreground">{title}</strong>
          {hint ? <small className="mt-0.5 block text-[9px] text-muted-foreground">{hint}</small> : null}
        </span>
        <motion.span
          aria-hidden
          animate={{ rotate: open ? 180 : 0 }}
          transition={reduce ? { duration: 0 } : SURFACE_SPRING}
          className="grid size-7 place-items-center rounded-full bg-muted text-muted-foreground"
        >
          <ChevronDown className="size-4" />
        </motion.span>
      </button>
      <motion.div
        aria-hidden={!open}
        inert={!open}
        initial={false}
        animate={{ height: open ? contentHeight : 0 }}
        transition={reduce ? { duration: 0 } : CONTENT_SPRING}
        className="overflow-hidden"
      >
        <motion.div
          ref={contentRef}
          initial={false}
          animate={{
            opacity: open ? 1 : 0,
            y: open || reduce ? 0 : -6,
            filter: open || reduce ? "blur(0px)" : "blur(4px)",
          }}
          transition={reduce ? { duration: 0 } : { duration: 0.2, ease: EASE_OUT }}
          className="border-t border-border px-5 pb-5"
        >
          {children}
        </motion.div>
      </motion.div>
    </motion.section>
  );
}
