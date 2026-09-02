import { describe, expect, it } from "vitest";
import { translate } from "./i18n";

describe("settings translations", () => {
  it("renders one selected language at a time", () => {
    expect(translate("zh-CN", "triggerTitle")).toBe("输入指定键盘指令时，播放宠物动画");
    expect(translate("en", "triggerTitle")).toBe(
      "Play a pet animation when a keyboard command is entered",
    );
  });

  it("interpolates pet counts", () => {
    expect(translate("zh-CN", "localPetCount", { count: 105 })).toBe("105 个宠物");
    expect(translate("en", "localPetCount", { count: 105 })).toBe("105 pets");
  });

  it("labels special trigger keys", () => {
    expect(translate("zh-CN", "spaceKey")).toBe("空格键");
    expect(translate("en", "enterKey")).toBe("Enter");
  });

  it("explains that observed shortcuts keep their original behavior", () => {
    expect(translate("zh-CN", "commandHint")).toContain("原操作照常执行");
    expect(translate("en", "commandHint")).toContain("observed passively");
  });

  it("localizes the bundled Spider-Man effects", () => {
    expect(translate("zh-CN", "spiderManPetName")).toBe("蜘蛛侠 4 · 动态彩蛋");
    expect(translate("en", "spiderManPetName")).toBe(
      "Spider-Man 4 · Animated Effects",
    );
  });
});
