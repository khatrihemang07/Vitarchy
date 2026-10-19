import { readLoadingDelay } from "./loadingDelay";

describe("readLoadingDelay", () => {
  afterEach(() => {
    document.documentElement.style.removeProperty("--duration-loading-delay");
  });

  it("reads the token from the document", () => {
    document.documentElement.style.setProperty("--duration-loading-delay", "250ms");
    expect(readLoadingDelay()).toBe(250);
  });

  it("is 0 when the token is not defined", () => {
    expect(readLoadingDelay()).toBe(0);
  });
});
