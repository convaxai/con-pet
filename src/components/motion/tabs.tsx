// Adapted from the MIT-licensed beUI Motion Tabs.
// https://beui.dev/components/motion/tabs
import { motion, MotionConfig, useReducedMotion } from "motion/react";
import {
  createContext,
  useCallback,
  useContext,
  useId,
  useMemo,
  type ReactNode,
} from "react";
import { SPRING_LAYOUT } from "@/lib/ease";
import { cn } from "@/lib/utils";

interface TabsContextValue {
  value: string;
  setValue: (value: string) => void;
  layoutId: string;
}

const TabsContext = createContext<TabsContextValue | null>(null);

function useTabs(): TabsContextValue {
  const context = useContext(TabsContext);
  if (!context) throw new Error("Tabs components must be used inside <Tabs>");
  return context;
}

export function Tabs({
  value,
  onValueChange,
  children,
  className,
}: {
  value: string;
  onValueChange: (value: string) => void;
  children: ReactNode;
  className?: string;
}) {
  const layoutId = useId();
  const reduce = useReducedMotion();
  const setValue = useCallback((next: string) => onValueChange(next), [onValueChange]);
  const context = useMemo(() => ({ value, setValue, layoutId }), [layoutId, setValue, value]);

  return (
    <MotionConfig transition={reduce ? { duration: 0 } : SPRING_LAYOUT}>
      <TabsContext.Provider value={context}>
        <motion.div layoutRoot className={className}>
          {children}
        </motion.div>
      </TabsContext.Provider>
    </MotionConfig>
  );
}

export function TabsList({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div
      role="tablist"
      className={cn("inline-flex items-center gap-0.5 rounded-full border border-border bg-card p-0.5", className)}
    >
      {children}
    </div>
  );
}

export function TabsTrigger({
  value,
  children,
  className,
}: {
  value: string;
  children: ReactNode;
  className?: string;
}) {
  const context = useTabs();
  const active = context.value === value;
  return (
    <div className="relative">
      {active ? (
        <motion.span
          layoutId={context.layoutId}
          layout="position"
          className="absolute inset-0 rounded-full bg-foreground"
        />
      ) : null}
      <button
        type="button"
        role="tab"
        aria-selected={active}
        onClick={() => context.setValue(value)}
        className={cn(
          "relative z-10 inline-flex h-7 min-w-11 items-center justify-center rounded-full px-2.5 text-[11px] font-medium outline-none transition-colors",
          active ? "text-background" : "text-muted-foreground hover:text-foreground",
          className,
        )}
      >
        {children}
      </button>
    </div>
  );
}
