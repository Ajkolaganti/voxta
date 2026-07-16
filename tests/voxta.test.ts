import { describe, expect, it } from "vitest";
import { defaultConfig } from "../src/services/voxta";

describe("Voxta frontend defaults", () => {
  it("starts with privacy-first defaults", () => {
    expect(defaultConfig.enabled).toBe(true);
    expect(defaultConfig.playSounds).toBe(true);
    expect(defaultConfig.spokenCommands).toBe(true);
    expect(defaultConfig.insertionMode).toBe("auto");
  });

  it("keeps onboarding incomplete until the user finishes first run", () => {
    expect(defaultConfig.onboardingComplete).toBe(false);
    expect(defaultConfig.launchAtLogin).toBe(false);
  });
});
