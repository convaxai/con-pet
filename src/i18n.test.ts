import { describe, expect, it } from "vitest";
import { translate } from "./i18n";

describe("settings translations", () => {
  it("renders one selected language at a time", () => {
    expect(translate("zh-CN", "triggerTitle")).toBe("键盘触发关键词时，播放宠物动画");
    expect(translate("en", "triggerTitle")).toBe(
      "Play a pet animation when the keyboard keyword is typed",
    );
  });

  it("interpolates pet counts", () => {
    expect(translate("zh-CN", "localPetCount", { count: 105 })).toBe("105 个宠物");
    expect(translate("en", "localPetCount", { count: 105 })).toBe("105 pets");
  });

  it("localizes the bundled Spider-Man effects", () => {
    expect(translate("zh-CN", "spiderManPetName")).toBe("蜘蛛侠 4 · 动态彩蛋");
    expect(translate("en", "spiderManPetName")).toBe(
      "Spider-Man 4 · Animated Effects",
    );
  });
});
