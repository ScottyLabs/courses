/* oxlint-disable typescript/no-floating-promises -- Courseweave's tests, ported unchanged (decision 14); node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { formatDays, formatTime, formatTimeRange, formatUnits } from "./time.ts";

describe("time formatting", () => {
  it("formats minutes as clock times", () => {
    expect(formatTime(570)).toBe("9:30 AM");
    expect(formatTime(720)).toBe("12:00 PM");
    expect(formatTime(0)).toBe("12:00 AM");
    expect(formatTime(1140, true)).toBe("7p");
    expect(formatTime(1170, true)).toBe("7:30p");
  });
  it("shares the AM/PM suffix in ranges when possible", () => {
    expect(formatTimeRange(570, 650)).toBe("9:30\u201310:50 AM");
    expect(formatTimeRange(660, 740)).toBe("11:00 AM\u201312:20 PM");
    expect(formatTimeRange(null, null)).toBe("TBA");
  });
  it("uses CMU day letters", () => {
    expect(formatDays([0, 2, 4])).toBe("MWF");
    expect(formatDays([1, 3])).toBe("TR");
    expect(formatDays([])).toBe("TBA");
  });
  it("labels variable units", () => {
    expect(formatUnits(null)).toBe("VAR");
    expect(formatUnits(4.5)).toBe("4.5");
  });
});
