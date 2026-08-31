// beui.dev/components/motion/select — MIT licensed, locally customized.

import { Check, ChevronDown } from "lucide-react";
import {
  motion,
  type Transition,
  useReducedMotion,
  type Variants,
} from "motion/react";
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { EASE_OUT } from "@/lib/ease";
import { cn } from "@/lib/utils";

const INSTANT_TRANSITION: Transition = { duration: 0 };
const CHEVRON_TRANSITION: Transition = {
  type: "spring",
  duration: 0.36,
  bounce: 0.22,
};
const LIST_VARIANTS: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.025, delayChildren: 0.035 } },
};
const ITEM_VARIANTS: Variants = {
  hidden: { opacity: 0, y: -4, filter: "blur(2px)" },
  show: { opacity: 1, y: 0, filter: "blur(0px)" },
};

type Placement = "bottom" | "top";

interface SelectContextValue {
  value: string | undefined;
  open: boolean;
  setOpen: (open: boolean) => void;
  select: (value: string) => void;
  register: (value: string, label: string) => void;
  unregister: (value: string) => void;
  labelFor: (value: string | undefined) => string | undefined;
  reduce: boolean;
  triggerId: string;
  listId: string;
  disabled: boolean;
  placement: Placement;
  setPlacement: (placement: Placement) => void;
}

const SelectContext = createContext<SelectContextValue | null>(null);

function useSelectContext(component: string): SelectContextValue {
  const context = useContext(SelectContext);
  if (!context) throw new Error(`${component} must be used within <Select>`);
  return context;
}

export interface SelectProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;
  className?: string;
  children: ReactNode;
}

export function Select({
  value,
  defaultValue,
  onValueChange,
  open: openProp,
  defaultOpen = false,
  onOpenChange,
  disabled = false,
  className,
  children,
}: SelectProps) {
  const reduce = useReducedMotion() ?? false;
  const baseId = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const [internalOpen, setInternalOpen] = useState(defaultOpen);
  const [internalValue, setInternalValue] = useState(defaultValue);
  const [labels, setLabels] = useState<Map<string, string>>(new Map());
  const [placement, setPlacement] = useState<Placement>("bottom");

  const controlled = value !== undefined;
  const current = controlled ? value : internalValue;
  const openControlled = openProp !== undefined;
  const open = openControlled ? openProp : internalOpen;

  const setOpen = useCallback(
    (next: boolean) => {
      if (!openControlled) setInternalOpen(next);
      onOpenChange?.(next);
    },
    [onOpenChange, openControlled],
  );

  const select = useCallback(
    (next: string) => {
      if (!controlled) setInternalValue(next);
      onValueChange?.(next);
      setOpen(false);
    },
    [controlled, onValueChange, setOpen],
  );

  const register = useCallback((itemValue: string, label: string) => {
    setLabels((currentLabels) =>
      currentLabels.get(itemValue) === label
        ? currentLabels
        : new Map(currentLabels).set(itemValue, label),
    );
  }, []);

  const unregister = useCallback((itemValue: string) => {
    setLabels((currentLabels) => {
      if (!currentLabels.has(itemValue)) return currentLabels;
      const next = new Map(currentLabels);
      next.delete(itemValue);
      return next;
    });
  }, []);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    const onPointerDown = (event: PointerEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) setOpen(false);
    };
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("pointerdown", onPointerDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("pointerdown", onPointerDown);
    };
  }, [open, setOpen]);

  const context = useMemo<SelectContextValue>(
    () => ({
      value: current,
      open,
      setOpen,
      select,
      register,
      unregister,
      labelFor: (itemValue) =>
        itemValue === undefined ? undefined : labels.get(itemValue),
      reduce,
      triggerId: `${baseId}-trigger`,
      listId: `${baseId}-list`,
      disabled,
      placement,
      setPlacement,
    }),
    [
      baseId,
      current,
      disabled,
      labels,
      open,
      placement,
      reduce,
      register,
      select,
      setOpen,
      unregister,
    ],
  );

  return (
    <SelectContext.Provider value={context}>
      <div ref={rootRef} className={cn("relative", className)} data-beui-select="">
        {children}
      </div>
    </SelectContext.Provider>
  );
}

export interface SelectTriggerProps {
  className?: string;
  children: ReactNode;
}

export function SelectTrigger({ className, children }: SelectTriggerProps) {
  const context = useSelectContext("SelectTrigger");
  const opensUp = context.placement === "top";
  const radiusFrames = context.open ? [0, 0, 20] : [20, 0, 20];
  const radiusTransition: Transition = context.reduce
    ? { duration: 0 }
    : { duration: context.open ? 0.58 : 0.42, times: [0, 0.42, 1], ease: EASE_OUT };

  return (
    <motion.button
      type="button"
      id={context.triggerId}
      disabled={context.disabled}
      aria-haspopup="listbox"
      aria-expanded={context.open}
      aria-controls={context.listId}
      onClick={() => context.setOpen(!context.open)}
      initial={false}
      animate={{
        borderTopLeftRadius: opensUp ? radiusFrames : 20,
        borderTopRightRadius: opensUp ? radiusFrames : 20,
        borderBottomLeftRadius: opensUp ? 20 : radiusFrames,
        borderBottomRightRadius: opensUp ? 20 : radiusFrames,
      }}
      transition={{
        borderTopLeftRadius: opensUp ? radiusTransition : INSTANT_TRANSITION,
        borderTopRightRadius: opensUp ? radiusTransition : INSTANT_TRANSITION,
        borderBottomLeftRadius: opensUp ? INSTANT_TRANSITION : radiusTransition,
        borderBottomRightRadius: opensUp ? INSTANT_TRANSITION : radiusTransition,
      }}
      className={cn(
        "relative z-10 flex h-10 w-full items-center justify-between gap-2 border border-border bg-background px-3 text-sm text-foreground outline-none",
        "transition-colors hover:border-border-strong focus-visible:ring-2 focus-visible:ring-ring/30",
        "disabled:pointer-events-none disabled:opacity-50",
        className,
      )}
    >
      {children}
      <motion.span
        aria-hidden
        animate={{ rotate: context.open ? 180 : 0 }}
        transition={context.reduce ? { duration: 0 } : CHEVRON_TRANSITION}
        className="text-muted-foreground"
      >
        <ChevronDown className="size-4" />
      </motion.span>
    </motion.button>
  );
}

export interface SelectValueProps {
  placeholder?: string;
  className?: string;
}

export function SelectValue({ placeholder, className }: SelectValueProps) {
  const context = useSelectContext("SelectValue");
  const label = context.labelFor(context.value);
  return (
    <span className={cn("truncate", label ? "text-foreground" : "text-muted-foreground", className)}>
      {label ?? placeholder ?? "Select"}
    </span>
  );
}

export interface SelectContentProps {
  className?: string;
  children: ReactNode;
}

export function SelectContent({ className, children }: SelectContentProps) {
  const context = useSelectContext("SelectContent");
  const innerRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(0);
  const { open, setPlacement } = context;

  useLayoutEffect(() => {
    const node = innerRef.current;
    if (!node) return;
    const measure = () => setHeight(node.offsetHeight);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(node);
    return () => observer.disconnect();
  });

  useLayoutEffect(() => {
    if (!open) return;
    const trigger = document.getElementById(context.triggerId);
    const node = innerRef.current;
    if (!trigger || !node) return;
    const rect = trigger.getBoundingClientRect();
    const below = window.innerHeight - rect.bottom;
    const above = rect.top;
    setPlacement(below < node.offsetHeight + 16 && above > below ? "top" : "bottom");
  }, [context.triggerId, open, setPlacement]);

  const opensUp = context.placement === "top";
  const nearGap = open ? 8 : 0;
  const nearRadius = open ? 16 : 0;
  const gapTransition: Transition = open
    ? { type: "spring", duration: 0.58, bounce: 0.42, delay: 0.1 }
    : { duration: 0.2 };

  return (
    <motion.div
      id={context.listId}
      role="listbox"
      aria-labelledby={context.triggerId}
      aria-hidden={!open}
      inert={!open}
      initial={false}
      animate={
        context.reduce
          ? { opacity: open ? 1 : 0, height: open ? height : 0 }
          : {
              opacity: open ? 1 : 0,
              height: open ? height : 0,
              marginTop: opensUp ? 0 : nearGap,
              marginBottom: opensUp ? nearGap : 0,
              borderTopLeftRadius: opensUp ? 16 : nearRadius,
              borderTopRightRadius: opensUp ? 16 : nearRadius,
              borderBottomLeftRadius: opensUp ? nearRadius : 16,
              borderBottomRightRadius: opensUp ? nearRadius : 16,
            }
      }
      transition={
        context.reduce
          ? { duration: 0.1 }
          : {
              opacity: { duration: 0.15 },
              height: open
                ? { type: "spring", duration: 0.36, bounce: 0.1 }
                : { duration: 0.2, ease: EASE_OUT },
              marginTop: opensUp ? INSTANT_TRANSITION : gapTransition,
              marginBottom: opensUp ? gapTransition : INSTANT_TRANSITION,
            }
      }
      style={{ overflow: "hidden", pointerEvents: open ? "auto" : "none" }}
      className={cn(
        "absolute right-0 left-0 z-50 border border-border bg-background shadow-[0_18px_50px_rgba(0,0,0,.5)]",
        opensUp ? "bottom-full" : "top-full",
        className,
      )}
    >
      <motion.div
        ref={innerRef}
        variants={context.reduce ? undefined : LIST_VARIANTS}
        initial={false}
        animate={open ? "show" : "hidden"}
        className="max-h-60 overflow-y-auto p-1"
      >
        {children}
      </motion.div>
    </motion.div>
  );
}

export interface SelectItemProps {
  value: string;
  disabled?: boolean;
  className?: string;
  children: ReactNode;
}

export function SelectItem({
  value,
  disabled = false,
  className,
  children,
}: SelectItemProps) {
  const context = useSelectContext("SelectItem");
  const selected = context.value === value;
  const label = typeof children === "string" ? children : value;

  useLayoutEffect(() => {
    context.register(value, label);
    return () => context.unregister(value);
  }, [context.register, context.unregister, label, value]);

  return (
    <motion.div variants={context.reduce ? undefined : ITEM_VARIANTS}>
      <button
        type="button"
        role="option"
        aria-selected={selected}
        disabled={disabled}
        onClick={() => context.select(value)}
        className={cn(
          "flex w-full items-center justify-between gap-2 rounded-lg px-2.5 py-2 text-left text-sm outline-none transition-colors",
          selected
            ? "bg-primary/12 text-primary"
            : "text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:bg-muted",
          "disabled:pointer-events-none disabled:opacity-50",
          className,
        )}
      >
        <span className="truncate">{children}</span>
        {selected ? <Check className="size-3.5 shrink-0" /> : null}
      </button>
    </motion.div>
  );
}
