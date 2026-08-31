import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { LogicalSize, PhysicalPosition } from "@tauri-apps/api/dpi";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  availableMonitors,
  cursorPosition,
  getCurrentWindow,
  primaryMonitor,
  type Monitor,
} from "@tauri-apps/api/window";
import { animations, randomTriggerAnimation } from "./animations";
import type { SpriteFrame } from "./animations";
import { computePosition } from "./positioning";
import type { AnimationName, AppConfig, PetPayload } from "./types";

const sleep = (milliseconds: number) =>
  new Promise<void>((resolve) => window.setTimeout(resolve, milliseconds));

const nextPaint = () =>
  new Promise<void>((resolve) =>
    window.requestAnimationFrame(() => window.requestAnimationFrame(() => resolve())),
  );

const decodedSpritesheets = new Map<string, Promise<void>>();

export async function renderOverlay(): Promise<void> {
  document.body.className = "overlay-body";
  document.body.innerHTML =
    '<div id="scene" aria-hidden="true"><div id="motion"><div id="sprite"></div></div></div>';
  const scene = document.querySelector<HTMLDivElement>("#scene");
  const motion = document.querySelector<HTMLDivElement>("#motion");
  const sprite = document.querySelector<HTMLDivElement>("#sprite");
  if (!scene || !motion || !sprite) return;

  const overlay = getCurrentWindow();
  // CSS transparency is required on every platform. These native calls also
  // clear the host window on macOS and the WebView surface on Windows.
  await overlay.setBackgroundColor([0, 0, 0, 0]);
  await getCurrentWebview().setBackgroundColor([0, 0, 0, 0]).catch(() => undefined);

  let generation = 0;
  let previousAnimation: AnimationName | undefined;
  await listen("keyword-triggered", async () => {
    const currentGeneration = ++generation;
    try {
      const [config, pet] = await Promise.all([
        invoke<AppConfig>("get_config"),
        invoke<PetPayload>("get_pet"),
      ]);
      if (!config.enabled) return;
      const animation = randomTriggerAnimation(pet.manifest.id, previousAnimation);
      previousAnimation = animation;
      await preloadSpritesheet(pet.spritesheetDataUrl);
      if (generation !== currentGeneration) return;
      // Always reconfigure while fully hidden. Without this, a rapid second
      // trigger can expose the previous atlas cell or an unpositioned sprite.
      await overlay.hide();
      scene.style.opacity = "0";
      await placeOverlay(config, pet, animation);
      configureSprite(motion, sprite, config, pet, animation);
      applyFrame(motion, sprite, pet, animations[animation].frames[0]);
      // The native command re-applies the macOS fullscreen-space behavior and
      // orders the overlay above the active app without taking keyboard focus.
      await invoke("show_overlay");
      await nextPaint();
      if (generation !== currentGeneration) return;
      scene.style.opacity = "1";
      await invoke("report_overlay_rendered");
      await play(
        motion,
        sprite,
        config,
        pet,
        animation,
        currentGeneration,
        () => generation,
      );
      if (generation === currentGeneration) {
        await overlay.hide();
        scene.style.opacity = "0";
      }
    } catch (error) {
      console.error("Con Pet overlay failed", error);
      await overlay.hide();
      scene.style.opacity = "0";
    }
  });
  await invoke("report_overlay_ready");
  await overlay.hide();
}

function configureSprite(
  motion: HTMLDivElement,
  sprite: HTMLDivElement,
  config: AppConfig,
  pet: PetPayload,
  animation: AnimationName,
): void {
  const padding = motionPadding(animation);
  motion.style.left = `${padding.x / 2}px`;
  motion.style.top = animation === "spider-heart" ? `${padding.y / 2}px` : "0";
  motion.style.width = `${pet.frameWidth * config.scale}px`;
  motion.style.height = `${pet.frameHeight * config.scale}px`;
  motion.style.transform = "none";
  motion.style.clipPath = "none";
  sprite.style.width = `${pet.frameWidth}px`;
  sprite.style.height = `${pet.frameHeight}px`;
  sprite.style.backgroundImage = `url("${pet.spritesheetDataUrl}")`;
  sprite.style.backgroundSize = `${pet.frameWidth * pet.columns}px ${pet.frameHeight * pet.rows}px`;
  sprite.style.transform = `scale(${config.scale})`;
}

function preloadSpritesheet(source: string): Promise<void> {
  const cached = decodedSpritesheets.get(source);
  if (cached) return cached;

  const loading = new Promise<void>((resolve, reject) => {
    const image = new Image();
    image.decoding = "sync";
    image.onload = () => {
      image.decode().then(() => resolve(), reject);
    };
    image.onerror = () => reject(new Error("序列帧图集加载失败"));
    image.src = source;
  });
  decodedSpritesheets.set(source, loading);
  loading.catch(() => decodedSpritesheets.delete(source));
  return loading;
}

function applyFrame(
  motion: HTMLDivElement,
  sprite: HTMLDivElement,
  pet: PetPayload,
  frame: SpriteFrame,
): void {
  sprite.style.backgroundPosition = `${-frame.column * pet.frameWidth}px ${-frame.row * pet.frameHeight}px`;
  motion.style.transformOrigin = frame.transformOrigin ?? "50% 0";
  motion.style.transform = `translate(${frame.offsetX ?? 0}px, ${frame.offsetY ?? 0}px) rotate(${frame.rotation ?? 0}deg)`;
  motion.style.clipPath =
    frame.reveal === undefined
      ? "none"
      : `inset(0 0 ${Number(((1 - frame.reveal) * 100).toFixed(2))}% 0)`;
}

async function play(
  motion: HTMLDivElement,
  sprite: HTMLDivElement,
  config: AppConfig,
  pet: PetPayload,
  animationName: AnimationName,
  generation: number,
  currentGeneration: () => number,
): Promise<void> {
  const animation = animations[animationName];
  for (let loop = 0; loop < config.loops; loop += 1) {
    for (const frame of animation.frames) {
      if (generation !== currentGeneration()) return;
      applyFrame(motion, sprite, pet, frame);
      await sleep(frame.duration);
    }
  }
}

async function placeOverlay(
  config: AppConfig,
  pet: PetPayload,
  animation: AnimationName,
): Promise<void> {
  const monitors = await availableMonitors();
  if (monitors.length === 0) return;
  const monitor = await chooseMonitor(config, monitors);
  const scaleFactor = monitor.scaleFactor;
  const baseWidth = pet.frameWidth;
  const padding = motionPadding(animation);
  const overlayWidth = baseWidth * config.scale + padding.x;
  const overlayHeight = pet.frameHeight * config.scale + padding.y;
  const spriteWidth = overlayWidth * scaleFactor;
  const spriteHeight = overlayHeight * scaleFactor;
  const position = computePosition({
    workArea: {
      x: monitor.workArea.position.x,
      y: monitor.workArea.position.y,
      width: monitor.workArea.size.width,
      height: monitor.workArea.size.height,
    },
    spriteWidth,
    spriteHeight,
    margin:
      animation === "spider-upside-down" ? 0 : config.position.margin * scaleFactor,
    mode: animation === "spider-upside-down" ? "top-center" : config.position.mode,
    fixedX: config.position.fixedX * scaleFactor,
    fixedY: config.position.fixedY * scaleFactor,
  });
  const overlay = getCurrentWindow();
  await overlay.setSize(new LogicalSize(overlayWidth, overlayHeight));
  await overlay.setPosition(new PhysicalPosition(position.x, position.y));
}

function motionPadding(animation: AnimationName): { x: number; y: number } {
  if (animation === "web-swing") return { x: 180, y: 52 };
  if (animation === "spider-upside-down") return { x: 0, y: 0 };
  if (animation === "spider-heart") return { x: 0, y: 12 };
  if (animation === "spider-crying") return { x: 12, y: 0 };
  return { x: 0, y: 0 };
}

async function chooseMonitor(config: AppConfig, monitors: Monitor[]): Promise<Monitor> {
  if (config.position.monitor === "random") {
    return monitors[Math.floor(Math.random() * monitors.length)];
  }
  if (config.position.monitor === "primary") {
    return (await primaryMonitor()) ?? monitors[0];
  }
  const cursor = await cursorPosition();
  return (
    monitors.find((monitor) => {
      const { position, size } = monitor;
      return (
        cursor.x >= position.x &&
        cursor.x < position.x + size.width &&
        cursor.y >= position.y &&
        cursor.y < position.y + size.height
      );
    }) ?? monitors[0]
  );
}
