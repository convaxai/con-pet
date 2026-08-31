import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ArrowLeft, Pause, Play, Search, SlidersHorizontal, Upload } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
  type ReactNode,
} from "react";
import { createRoot } from "react-dom/client";
import {
  checkInputMonitoringPermission,
} from "tauri-plugin-macos-permissions-api";
import { animations, galleryPreviewAnimation } from "./animations";
import {
  AnimatedBadge,
  type AnimatedBadgeStatus,
} from "./components/motion/animated-badge";
import { BouncyAccordion } from "./components/motion/bouncy-accordion";
import { Button } from "./components/motion/button";
import { Input } from "./components/motion/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./components/motion/select";
import { Switch } from "./components/motion/switch";
import { Tabs, TabsList, TabsTrigger } from "./components/motion/tabs";
import { translate, type Locale } from "./i18n";
import { cn } from "./lib/utils";
import type {
  AnimationName,
  AppConfig,
  MarketInstallResult,
  MonitorMode,
  PetChoice,
  PetPayload,
  PetThumbnail,
  PositionMode,
  RuntimeStatus,
} from "./types";

interface LocalPet {
  manifestPath: string | null;
  id: string;
  displayName: string;
  description: string;
}

interface Notice {
  message: string;
  kind: "success" | "error";
}

interface SelectOption {
  value: string;
  label: string;
}

const thumbnailCache = new Map<string, string>();
const thumbnailRequests = new Map<string, Promise<string>>();
const payloadCache = new Map<string, Promise<PetPayload>>();
const PAYLOAD_CACHE_LIMIT = 12;

function petCacheKey(pet: LocalPet): string {
  return pet.manifestPath ?? "builtin:spiderman4";
}

async function checkPermission(platform: string): Promise<boolean> {
  if (platform !== "macos") return true;
  try {
    return await checkInputMonitoringPermission();
  } catch {
    return false;
  }
}

async function loadThumbnail(pet: LocalPet): Promise<string> {
  const key = petCacheKey(pet);
  const cached = thumbnailCache.get(key);
  if (cached) return cached;
  const active = thumbnailRequests.get(key);
  if (active) return active;
  const request = invoke<PetThumbnail>("get_pet_thumbnail", {
    manifestPath: pet.manifestPath,
  }).then((thumbnail) => {
    thumbnailCache.set(key, thumbnail.imageDataUrl);
    thumbnailRequests.delete(key);
    return thumbnail.imageDataUrl;
  });
  thumbnailRequests.set(key, request);
  return request;
}

function loadPetPayload(pet: LocalPet): Promise<PetPayload> {
  const key = petCacheKey(pet);
  const cached = payloadCache.get(key);
  if (cached) {
    payloadCache.delete(key);
    payloadCache.set(key, cached);
    return cached;
  }

  const request = invoke<PetPayload>("get_pet_preview", {
    manifestPath: pet.manifestPath,
  });
  payloadCache.set(key, request);
  while (payloadCache.size > PAYLOAD_CACHE_LIMIT) {
    const oldestKey = payloadCache.keys().next().value;
    if (oldestKey === undefined) break;
    payloadCache.delete(oldestKey);
  }
  void request.catch(() => {
    if (payloadCache.get(key) === request) payloadCache.delete(key);
  });
  return request;
}

async function discoverPets(locale: Locale, selectedPath: string | null): Promise<LocalPet[]> {
  const choices = await invoke<PetChoice[]>("list_codex_pets");
  const pets: LocalPet[] = [
    {
      manifestPath: null,
      id: "spiderman4-sticker",
      displayName: translate(locale, "spiderManPetName"),
      description: translate(locale, "spiderManPetDescription"),
    },
    ...choices.filter((choice) => choice.id !== "spiderman4-sticker").map((choice) => {
      if (choice.id === "pathlight") {
        return { ...choice, displayName: translate(locale, "builtinPathlight") };
      }
      return { ...choice };
    }),
  ];
  if (
    selectedPath &&
    selectedPath !== "builtin:spiderman4" &&
    !pets.some((pet) => pet.manifestPath === selectedPath)
  ) {
    pets.push({
      manifestPath: selectedPath,
      id: "custom",
      displayName: translate(locale, "customPet"),
      description: "pet.json",
    });
  }
  return pets;
}

export async function renderSettings(): Promise<void> {
  document.body.className = "settings-body";
  createRoot(document.body).render(<SettingsApp />);
}

function SettingsApp() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [draft, setDraft] = useState<AppConfig | null>(null);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [autostartBusy, setAutostartBusy] = useState(false);
  const [inputPermissionGranted, setInputPermissionGranted] = useState(false);
  const [pets, setPets] = useState<LocalPet[]>([]);
  const [page, setPage] = useState<"settings" | "pets">("settings");
  const [query, setQuery] = useState("");
  const [notice, setNotice] = useState<Notice | null>(null);
  const [openSelect, setOpenSelect] = useState<string | null>(null);
  const [petBusy, setPetBusy] = useState<string | null>(null);
  const [importBusy, setImportBusy] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  const locale = draft?.locale ?? config?.locale ?? "zh-CN";
  const tx = useCallback(
    (
      key: Parameters<typeof translate>[1],
      values?: Record<string, string | number>,
    ) => translate(locale, key, values),
    [locale],
  );

  const notify = useCallback((message: string, kind: Notice["kind"] = "success") => {
    setNotice({ message, kind });
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), 3200);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const refreshPets = useCallback(async (nextLocale: Locale, selectedPath: string | null) => {
    setPets(await discoverPets(nextLocale, selectedPath));
  }, []);

  useEffect(() => {
    let active = true;
    const unlisteners: UnlistenFn[] = [];

    void Promise.all([
      invoke<AppConfig>("get_config"),
      invoke<RuntimeStatus>("get_runtime_status"),
      invoke<boolean>("get_autostart_enabled"),
    ]).then(async ([initialConfig, initialRuntime, initialAutostart]) => {
      if (!active) return;
      setConfig(initialConfig);
      setDraft(initialConfig);
      setRuntime(initialRuntime);
      setAutostartEnabled(initialAutostart);
      document.documentElement.lang = initialConfig.locale;
      setInputPermissionGranted(await checkPermission(initialRuntime.platform));
      await refreshPets(initialConfig.locale, initialConfig.petManifestPath);
    });

    void Promise.all([
      listen<boolean>("pause-changed", ({ payload }) => {
        setRuntime((current) => (current ? { ...current, paused: payload } : current));
      }),
      listen<string>("listener-error", ({ payload }) => {
        setRuntime((current) =>
          current ? { ...current, listenerError: payload, listenerRunning: false } : current,
        );
      }),
      listen<boolean>("autostart-changed", ({ payload }) => {
        setAutostartEnabled(payload);
      }),
      listen<AppConfig>("config-changed", ({ payload }) => {
        setConfig(payload);
        setDraft(payload);
        document.documentElement.lang = payload.locale;
        void refreshPets(payload.locale, payload.petManifestPath);
        notify(translate(payload.locale, "synced"));
      }),
    ]).then((items) => unlisteners.push(...items));

    return () => {
      active = false;
      for (const unlisten of unlisteners) unlisten();
    };
  }, [notify, refreshPets]);

  useEffect(() => {
    if (!runtime) return;
    const interval = window.setInterval(() => {
      void Promise.all([
        invoke<RuntimeStatus>("get_runtime_status"),
        checkPermission(runtime.platform),
      ]).then(([nextRuntime, permission]) => {
        setRuntime(nextRuntime);
        setInputPermissionGranted(permission);
      });
    }, 800);
    return () => window.clearInterval(interval);
  }, [runtime?.platform]);

  const persist = useCallback(async (): Promise<AppConfig> => {
    if (!draft) throw new Error("Settings are not ready");
    const saved = await invoke<AppConfig>("save_config", { config: draft });
    setConfig(saved);
    setDraft(saved);
    return saved;
  }, [draft]);

  const handleSave = async (event?: FormEvent) => {
    event?.preventDefault();
    try {
      await persist();
      notify(tx("saved"));
    } catch (error) {
      notify(String(error), "error");
    }
  };

  const handlePreview = async () => {
    try {
      await persist();
      await invoke("test_trigger");
    } catch (error) {
      notify(String(error), "error");
    }
  };

  const handleLocale = async (nextLocale: string) => {
    if (!draft) return;
    try {
      const nextDraft = { ...draft, locale: nextLocale as Locale };
      const saved = await invoke<AppConfig>("save_config", { config: nextDraft });
      setConfig(saved);
      setDraft(saved);
      document.documentElement.lang = saved.locale;
      await refreshPets(saved.locale, saved.petManifestPath);
    } catch (error) {
      notify(String(error), "error");
    }
  };

  const handlePause = async () => {
    if (!runtime) return;
    const paused = await invoke<boolean>("set_paused", { paused: !runtime.paused });
    setRuntime({ ...runtime, paused });
  };

  const handleAutostart = async (enabled: boolean) => {
    setAutostartBusy(true);
    try {
      const next = await invoke<boolean>("set_autostart_enabled", { enabled });
      setAutostartEnabled(next);
      notify(next ? tx("autostartEnabled") : tx("autostartDisabled"));
    } catch (error) {
      notify(String(error), "error");
    } finally {
      setAutostartBusy(false);
    }
  };

  const handlePermission = async () => {
    try {
      await invoke<boolean>("request_input_monitoring_access");
      notify(tx("permissionRequested"));
      window.setTimeout(() => {
        void checkPermission("macos").then(setInputPermissionGranted);
      }, 600);
    } catch (error) {
      notify(String(error), "error");
    }
  };

  const applyPet = async (pet: LocalPet, selectedNotice = true): Promise<AppConfig> => {
    const previousPet = pets.find((candidate) => candidate.manifestPath === config?.petManifestPath);
    let selected = await invoke<AppConfig>("select_pet", {
      manifestPath: pet.manifestPath,
    });
    if (pet.id === "spiderman4-sticker") {
      selected = await invoke<AppConfig>("save_config", {
        config: {
          ...selected,
          loops: 1,
          scale: 1.6,
          position: { ...selected.position, mode: "top-center", margin: 0 },
        },
      });
      if (selectedNotice) notify(tx("spidermanSelected"));
    } else {
      if (previousPet?.id === "spiderman4-sticker") {
        selected = await invoke<AppConfig>("save_config", {
          config: { ...selected, loops: 2 },
        });
      }
      if (selectedNotice) notify(tx("petSelected"));
    }
    setConfig(selected);
    setDraft(selected);
    await refreshPets(selected.locale, selected.petManifestPath);
    return selected;
  };

  const handlePet = async (pet: LocalPet) => {
    setPetBusy(pet.id);
    try {
      await applyPet(pet);
    } catch (error) {
      notify(String(error), "error");
    } finally {
      setPetBusy(null);
    }
  };

  const handleImport = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Codex Pet ZIP", extensions: ["zip"] }],
    });
    if (typeof selected !== "string") return;
    setImportBusy(true);
    try {
      const result = await invoke<MarketInstallResult>("import_pet_zip", {
        archivePath: selected,
      });
      const importedPet: LocalPet = {
        manifestPath: result.manifestPath,
        id: result.id,
        displayName: result.id,
        description: "",
      };
      await applyPet(importedPet, false);
      notify(result.alreadyInstalled ? tx("importExisting") : tx("importSuccess"));
    } catch (error) {
      notify(String(error), "error");
    } finally {
      setImportBusy(false);
    }
  };

  const showPage = (nextPage: "settings" | "pets") => {
    setOpenSelect(null);
    setQuery("");
    setPage(nextPage);
    document.body.scrollTo({ top: 0 });
  };

  if (!draft || !runtime || !config) {
    return (
      <main className="loading-shell">
        <span className="app-icon">C</span>
        <strong>Con Pet</strong>
      </main>
    );
  }

  const permissionMissing = runtime.platform === "macos" && !inputPermissionGranted;
  const healthy = runtime.listenerRunning && !runtime.listenerError && !permissionMissing;
  const listenerLabel = permissionMissing
    ? tx("listenerAwaitingPermission")
    : runtime.listenerError
      ? tx("listenerFailed")
      : runtime.paused
        ? tx("listenerPaused")
        : healthy
          ? tx("listenerActive")
          : tx("listenerStarting");
  const listenerBadgeStatus: AnimatedBadgeStatus = permissionMissing
    ? "warning"
    : runtime.listenerError
      ? "danger"
      : runtime.paused
        ? "neutral"
        : healthy
          ? "success"
          : "loading";
  const currentPet =
    pets.find((pet) => pet.manifestPath === config.petManifestPath) ?? pets[0];

  if (page === "pets") {
    return (
      <PetGallery
        locale={locale}
        pets={pets}
        selectedPath={config.petManifestPath}
        query={query}
        onQueryChange={setQuery}
        onBack={() => showPage("settings")}
        onImport={() => void handleImport()}
        importBusy={importBusy}
        petBusy={petBusy}
        onSelect={(pet) => void handlePet(pet)}
      />
    );
  }

  const positionOptions: SelectOption[] = [
    { value: "random", label: tx("positionRandom") },
    { value: "top-left", label: tx("positionTopLeft") },
    { value: "top-center", label: tx("positionTopCenter") },
    { value: "top-right", label: tx("positionTopRight") },
    { value: "bottom-left", label: tx("positionBottomLeft") },
    { value: "bottom-right", label: tx("positionBottomRight") },
    { value: "fixed", label: tx("positionFixed") },
  ];
  const monitorOptions: SelectOption[] = [
    { value: "cursor", label: tx("monitorCursor") },
    { value: "primary", label: tx("monitorPrimary") },
    { value: "random", label: tx("monitorRandom") },
  ];

  return (
    <>
      <main className="app-shell">
        <header className="app-header">
          <div className="app-title">
            <span className="app-icon">C</span>
            <h1>Con Pet</h1>
          </div>
          <div className="header-controls">
            <span className="status-copy">{tx("keywordStatus", { keyword: draft.keyword })}</span>
            <AnimatedBadge status={listenerBadgeStatus}>
              {listenerLabel}
            </AnimatedBadge>
            <Button
              size="icon"
              aria-label={runtime.paused ? tx("resume") : tx("pause")}
              onClick={() => void handlePause()}
            >
              {runtime.paused ? <Play className="size-3.5" /> : <Pause className="size-3.5" />}
            </Button>
            <Tabs
              value={locale}
              onValueChange={(value) => void handleLocale(value)}
              className="language-tabs"
            >
              <TabsList>
                <TabsTrigger value="zh-CN">中</TabsTrigger>
                <TabsTrigger value="en">EN</TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
        </header>

        {runtime.platform === "macos" && !inputPermissionGranted ? (
          <section className="permission-card">
            <div>
              <strong>{tx("permissionTitle")}</strong>
              <p>{tx("permissionHelp")}</p>
            </div>
            <AnimatedBadge status="warning">{tx("needsPermission")}</AnimatedBadge>
            <Button variant="outline" ripple onClick={() => void handlePermission()}>
              {tx("openSystemSettings")}
            </Button>
          </section>
        ) : null}

        <form className="settings-form" onSubmit={(event) => void handleSave(event)}>
          <div className="settings-scroll">
          <section className="card">
            <div className="section-heading">
              <h2>{tx("triggerTitle")}</h2>
              <Switch
                checked={draft.enabled}
                onCheckedChange={(enabled) => setDraft({ ...draft, enabled })}
                label={tx("enabled")}
              />
            </div>
            <FormField label={tx("keyword")} hint={tx("keywordHint")}>
              <Input
                value={draft.keyword}
                maxLength={64}
                autoComplete="off"
                spellCheck={false}
                placeholder={tx("keywordPlaceholder")}
                onValueChange={(keyword) => setDraft({ ...draft, keyword })}
              />
            </FormField>
          </section>

          <section className="card pet-choice-card">
            <div className="pet-choice-current">
              {currentPet ? <AnimationFigure pet={currentPet} /> : null}
              <div className="pet-choice-copy">
                <span className="section-label">{tx("currentPet")}</span>
                <h2>{currentPet?.displayName ?? tx("loading")}</h2>
              </div>
            </div>
            <div className="pet-choice-actions">
              <span className="muted">{tx("localPetCount", { count: pets.length })}</span>
              <Button variant="secondary" ripple onClick={() => showPage("pets")}>
                {tx("choosePet")}
              </Button>
            </div>
          </section>

          <section className="card">
            <div className="section-heading">
              <h2>{tx("appearance")}</h2>
            </div>
            <div className="field-grid two">
              <FormField label={tx("size")}>
                <Input
                  type="number"
                  min={0.4}
                  max={2.5}
                  step={0.1}
                  value={draft.scale}
                  onValueChange={(value) => setDraft({ ...draft, scale: Number(value) })}
                />
              </FormField>
              <FormField label={tx("position")}>
                <MotionSelect
                  id="position"
                  value={draft.position.mode}
                  options={positionOptions}
                  openSelect={openSelect}
                  setOpenSelect={setOpenSelect}
                  onValueChange={(value) =>
                    setDraft({
                      ...draft,
                      position: { ...draft.position, mode: value as PositionMode },
                    })
                  }
                />
              </FormField>
            </div>
          </section>

          <section className="card system-card">
            <div className="section-heading system-heading">
              <div>
                <h2>{tx("background")}</h2>
                <p>{tx("backgroundHint")}</p>
              </div>
              <Switch
                checked={autostartEnabled}
                disabled={autostartBusy}
                onCheckedChange={(enabled) => void handleAutostart(enabled)}
                label={tx("launchAtLogin")}
              />
            </div>
          </section>

          <BouncyAccordion
            title={tx("advanced")}
            hint={tx("advancedHint")}
            icon={<SlidersHorizontal className="size-3.5" />}
            open={advancedOpen}
            onOpenChange={setAdvancedOpen}
            className="advanced-settings"
          >
            <div className="advanced-content">
              <div className="field-grid two">
                <FormField label={tx("sequenceTimeout")}>
                  <Input
                    type="number"
                    min={500}
                    max={30000}
                    step={100}
                    value={draft.sequenceTimeoutMs}
                    onValueChange={(value) =>
                      setDraft({ ...draft, sequenceTimeoutMs: Number(value) })
                    }
                  />
                </FormField>
                <FormField label={tx("cooldown")}>
                  <Input
                    type="number"
                    min={250}
                    max={60000}
                    value={draft.cooldownMs}
                    onValueChange={(value) =>
                      setDraft({ ...draft, cooldownMs: Number(value) })
                    }
                  />
                </FormField>
                <FormField label={tx("loops")}>
                  <Input
                    type="number"
                    min={1}
                    max={10}
                    value={draft.loops}
                    onValueChange={(value) =>
                      setDraft({ ...draft, loops: Number(value) })
                    }
                  />
                </FormField>
                <FormField label={tx("monitor")}>
                  <MotionSelect
                    id="monitor"
                    value={draft.position.monitor}
                    options={monitorOptions}
                    openSelect={openSelect}
                    setOpenSelect={setOpenSelect}
                    onValueChange={(value) =>
                      setDraft({
                        ...draft,
                        position: { ...draft.position, monitor: value as MonitorMode },
                      })
                    }
                  />
                </FormField>
                <FormField label={tx("margin")}>
                  <Input
                    type="number"
                    min={0}
                    max={400}
                    value={draft.position.margin}
                    onValueChange={(value) =>
                      setDraft({
                        ...draft,
                        position: { ...draft.position, margin: Number(value) },
                      })
                    }
                  />
                </FormField>
              </div>
              {draft.position.mode === "fixed" ? (
                <div className="field-grid two">
                  <FormField label={tx("xCoordinate")}>
                    <Input
                      type="number"
                      value={draft.position.fixedX}
                      onValueChange={(value) =>
                        setDraft({
                          ...draft,
                          position: { ...draft.position, fixedX: Number(value) },
                        })
                      }
                    />
                  </FormField>
                  <FormField label={tx("yCoordinate")}>
                    <Input
                      type="number"
                      value={draft.position.fixedY}
                      onValueChange={(value) =>
                        setDraft({
                          ...draft,
                          position: { ...draft.position, fixedY: Number(value) },
                        })
                      }
                    />
                  </FormField>
                </div>
              ) : null}
            </div>
          </BouncyAccordion>
          </div>

          <footer className="actions">
            <Button variant="outline" ripple onClick={() => void handlePreview()}>
              {tx("testAnimation")}
            </Button>
            <Button type="submit" variant="primary" ripple>
              {tx("save")}
            </Button>
          </footer>
        </form>
      </main>
      {notice ? <NoticeToast notice={notice} /> : null}
    </>
  );
}

function FormField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="form-field">
      <span className="form-label">{label}</span>
      {children}
      {hint ? <small>{hint}</small> : null}
    </div>
  );
}

function MotionSelect({
  id,
  value,
  options,
  openSelect,
  setOpenSelect,
  onValueChange,
  compact = false,
}: {
  id: string;
  value: string;
  options: SelectOption[];
  openSelect: string | null;
  setOpenSelect: (id: string | null) => void;
  onValueChange: (value: string) => void;
  compact?: boolean;
}) {
  const isOpen = openSelect === id;
  return (
    <Select
      value={value}
      open={isOpen}
      onOpenChange={(openState) => setOpenSelect(openState ? id : null)}
      onValueChange={onValueChange}
      className={cn(isOpen && "z-50", compact && "w-28")}
    >
      <SelectTrigger className={compact ? "h-8 px-2.5 text-xs" : undefined}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {options.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

function AnimationFigure({ pet }: { pet: LocalPet }) {
  const [payload, setPayload] = useState<PetPayload | null>(null);
  const animation = galleryPreviewAnimation(pet.id);

  useEffect(() => {
    let active = true;
    setPayload(null);
    void loadPetPayload(pet)
      .then((nextPayload) => {
        if (active) setPayload(nextPayload);
      })
      .catch(() => {
        if (active) setPayload(null);
      });
    return () => {
      active = false;
    };
  }, [pet.manifestPath]);

  return (
    <div className="animation-figure" role="img" aria-label={pet.displayName}>
      <div className="animation-figure-visual">
        {payload ? (
          <SpriteAnimation payload={payload} animation={animation} playing maxScale={0.28} />
        ) : (
          <span className="pet-gallery-placeholder">P</span>
        )}
      </div>
    </div>
  );
}

function NoticeToast({ notice }: { notice: Notice }) {
  const reduce = useReducedMotion();
  return (
    <motion.div
      role="status"
      initial={reduce ? { opacity: 0 } : { opacity: 0, y: 8, scale: 0.98 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      className={cn("notice", notice.kind === "error" && "error")}
    >
      {notice.message}
    </motion.div>
  );
}

function PetGallery({
  locale,
  pets,
  selectedPath,
  query,
  onQueryChange,
  onBack,
  onImport,
  importBusy,
  petBusy,
  onSelect,
}: {
  locale: Locale;
  pets: LocalPet[];
  selectedPath: string | null;
  query: string;
  onQueryChange: (query: string) => void;
  onBack: () => void;
  onImport: () => void;
  importBusy: boolean;
  petBusy: string | null;
  onSelect: (pet: LocalPet) => void;
}) {
  const tx = useCallback(
    (key: Parameters<typeof translate>[1]) => translate(locale, key),
    [locale],
  );
  const normalized = query.trim().toLocaleLowerCase();
  const visiblePets = useMemo(
    () =>
      normalized
        ? pets.filter((pet) =>
            `${pet.displayName} ${pet.description}`.toLocaleLowerCase().includes(normalized),
          )
        : pets,
    [normalized, pets],
  );

  return (
    <main className="pets-page">
      <div className="pets-sticky">
        <header className="pets-toolbar-row">
          <Button size="icon" aria-label={tx("back")} onClick={onBack}>
            <ArrowLeft className="size-4" />
          </Button>
          <h1>{tx("galleryTitle")}</h1>
          <div className="pets-search">
            <Input
              type="search"
              value={query}
              leftIcon={<Search />}
              placeholder={tx("searchPets")}
              autoComplete="off"
              onValueChange={onQueryChange}
            />
          </div>
          <Button variant="outline" ripple disabled={importBusy} onClick={onImport}>
            <Upload className="size-4" />
            {importBusy ? tx("importing") : tx("importZip")}
          </Button>
        </header>
      </div>
      <section className="pet-view">
        <div className="pet-view-heading">
          <h2>{tx("gallerySection")}</h2>
        </div>
        <div className="pet-gallery-grid">
          {visiblePets.length ? (
            visiblePets.map((pet) => (
              <PetCard
                key={`${pet.id}:${pet.manifestPath ?? "builtin"}`}
                pet={pet}
                active={pet.manifestPath === selectedPath}
                busy={petBusy === pet.id}
                useLabel={tx("use")}
                inUseLabel={tx("inUse")}
                onSelect={() => onSelect(pet)}
              />
            ))
          ) : (
            <p className="pet-empty">{tx("noMatches")}</p>
          )}
        </div>
      </section>
    </main>
  );
}

function PetCard({
  pet,
  active,
  busy,
  useLabel,
  inUseLabel,
  onSelect,
}: {
  pet: LocalPet;
  active: boolean;
  busy: boolean;
  useLabel: string;
  inUseLabel: string;
  onSelect: () => void;
}) {
  const reduce = useReducedMotion();
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const previewing = hovered || focused;
  return (
    <motion.button
      type="button"
      aria-pressed={active}
      disabled={busy}
      whileTap={reduce ? undefined : { scale: 0.985 }}
      whileHover={reduce ? undefined : { y: -3, scale: 1.008 }}
      onHoverStart={() => setHovered(true)}
      onHoverEnd={() => setHovered(false)}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      onClick={onSelect}
      className={cn("pet-gallery-card", active && "active")}
    >
      <span className="pet-gallery-visual">
        <span className="pet-gallery-placeholder">P</span>
        <PetImage pet={pet} previewing={previewing} />
      </span>
      <span className="pet-gallery-copy">
        <strong>{pet.displayName}</strong>
      </span>
      <span className="pet-gallery-action">{active ? inUseLabel : useLabel}</span>
    </motion.button>
  );
}

function PetImage({ pet, previewing }: { pet: LocalPet; previewing: boolean }) {
  const ref = useRef<HTMLImageElement>(null);
  const [source, setSource] = useState<string | null>(
    thumbnailCache.get(petCacheKey(pet)) ?? null,
  );
  const [payload, setPayload] = useState<PetPayload | null>(null);

  useEffect(() => {
    const image = ref.current;
    if (!image || source) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        observer.disconnect();
        void loadThumbnail(pet).then(setSource).catch(() => setSource(null));
      },
      { rootMargin: "180px" },
    );
    observer.observe(image);
    return () => observer.disconnect();
  }, [pet, source]);

  useEffect(() => {
    if (!previewing || payload) return;
    let active = true;
    void loadPetPayload(pet)
      .then((nextPayload) => {
        if (active) setPayload(nextPayload);
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, [payload, pet, previewing]);

  return (
    <>
      <img
        ref={ref}
        src={source ?? undefined}
        className={cn(source && "ready", payload && previewing && "preview-hidden")}
        alt={pet.displayName}
        loading="lazy"
      />
      {payload && previewing ? (
        <SpriteAnimation
          payload={payload}
          animation={galleryPreviewAnimation(pet.id)}
          playing
          maxScale={0.7}
        />
      ) : null}
    </>
  );
}

function SpriteAnimation({
  payload,
  animation,
  playing,
  maxScale,
}: {
  payload: PetPayload;
  animation: AnimationName;
  playing: boolean;
  maxScale: number;
}) {
  const reduce = useReducedMotion();
  const previewRef = useRef<HTMLSpanElement>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [viewport, setViewport] = useState({ width: 0, height: 0 });
  const frames = animations[animation].frames;
  const bounds = useMemo(() => {
    let minX = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (const candidate of frames) {
      const radians = ((candidate.rotation ?? 0) * Math.PI) / 180;
      const rotatedWidth =
        Math.abs(payload.frameWidth * Math.cos(radians)) +
        Math.abs(payload.frameHeight * Math.sin(radians));
      const rotatedHeight =
        Math.abs(payload.frameWidth * Math.sin(radians)) +
        Math.abs(payload.frameHeight * Math.cos(radians));
      const x = candidate.offsetX ?? 0;
      const y = candidate.offsetY ?? 0;
      minX = Math.min(minX, x - rotatedWidth / 2);
      maxX = Math.max(maxX, x + rotatedWidth / 2);
      minY = Math.min(minY, y - rotatedHeight / 2);
      maxY = Math.max(maxY, y + rotatedHeight / 2);
    }
    return {
      width: maxX - minX,
      height: maxY - minY,
      centerX: (minX + maxX) / 2,
      centerY: (minY + maxY) / 2,
    };
  }, [frames, payload.frameHeight, payload.frameWidth]);

  useEffect(() => {
    const preview = previewRef.current;
    if (!preview) return;
    const updateSize = () => {
      const { width, height } = preview.getBoundingClientRect();
      setViewport((current) =>
        current.width === width && current.height === height ? current : { width, height },
      );
    };
    updateSize();
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(updateSize);
    observer.observe(preview);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    setFrameIndex(0);
    if (!playing || reduce || frames.length < 2) return;

    let index = 0;
    let timeout = 0;
    const advance = () => {
      timeout = window.setTimeout(() => {
        index = (index + 1) % frames.length;
        setFrameIndex(index);
        advance();
      }, frames[index].duration);
    };
    advance();
    return () => window.clearTimeout(timeout);
  }, [animation, frames, playing, reduce]);

  const frame = frames[frameIndex] ?? frames[0];
  const padding = 8;
  const scale =
    viewport.width > 0 && viewport.height > 0
      ? Math.min(
          maxScale,
          Math.max(0.01, viewport.width - padding * 2) / bounds.width,
          Math.max(0.01, viewport.height - padding * 2) / bounds.height,
        )
      : maxScale;
  const offsetX = ((frame.offsetX ?? 0) - bounds.centerX) * scale;
  const offsetY = ((frame.offsetY ?? 0) - bounds.centerY) * scale;
  return (
    <span ref={previewRef} className="pet-sprite-preview" aria-hidden="true">
      <span className="pet-sprite-motion">
        <span
          className="pet-sprite-frame"
          style={{
            width: `${payload.frameWidth}px`,
            height: `${payload.frameHeight}px`,
            backgroundImage: `url("${payload.spritesheetDataUrl}")`,
            backgroundSize: `${payload.frameWidth * payload.columns}px ${payload.frameHeight * payload.rows}px`,
            backgroundPosition: `${-frame.column * payload.frameWidth}px ${-frame.row * payload.frameHeight}px`,
            transform: `translate(calc(-50% + ${offsetX}px), calc(-50% + ${offsetY}px)) rotate(${frame.rotation ?? 0}deg) scale(${scale})`,
          }}
        />
      </span>
    </span>
  );
}
