/** @vitest-environment jsdom */
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { StatusBar } from "./StatusBar";

declare global {
  var IS_REACT_ACT_ENVIRONMENT: boolean;
}

let host: HTMLDivElement;
let root: Root;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
});

afterEach(() => {
  act(() => root.unmount());
  host.remove();
});

it("opens the application log when the name and version are right-clicked", () => {
  const onOpenLog = vi.fn();
  act(() => root.render(<StatusBar version="1.2.3" onOpenLog={onOpenLog} />));

  const logo = host.querySelector<HTMLElement>("footer > span");
  expect(logo?.textContent).toBe("rbxport 1.2.3");
  const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
  act(() => {
    logo?.dispatchEvent(event);
  });

  expect(event.defaultPrevented).toBe(true);
  expect(onOpenLog).toHaveBeenCalledOnce();
});

it("names read-only as a library state", () => {
  act(() => root.render(<StatusBar readOnly />));

  const badge = host.querySelector<HTMLButtonElement>("footer > button");
  expect(badge?.textContent).toBe("Library read-only");
  expect(badge?.title).toContain("Editing is locked while rekordbox is running");
});

it("offers the support action when it is available", () => {
  const onSupport = vi.fn();
  act(() => root.render(<StatusBar onSupport={onSupport} />));

  const button = Array.from(host.querySelectorAll("button")).find(candidate => candidate.textContent?.includes("Support rbxport"));
  act(() => button?.click());

  expect(button).toBeDefined();
  expect(onSupport).toHaveBeenCalledOnce();
});

it("lists the running job and then those waiting, each with its own Stop", () => {
  const onStopJob = vi.fn();
  act(() => root.render(<StatusBar onStopJob={onStopJob} jobs={[
    { id: 1, label: "Adding 495 to Set", done: 99, total: 495, state: "running", target: "p", pendingRows: 396 },
    { id: 2, label: "Removing 10 from the collection", done: 0, total: 10, state: "queued", target: null, pendingRows: 0 },
  ]} />));
  const meters = [...host.querySelectorAll("[data-state]")];
  expect(meters.map((m) => m.getAttribute("data-state"))).toEqual(["running", "queued"]);
  expect(meters[0]?.textContent).toContain("(20%)");
  expect(meters[1]?.textContent).toContain("Queued");
  const stops = host.querySelectorAll<HTMLButtonElement>("button[aria-label^='Stop']");
  expect(stops).toHaveLength(2);
  act(() => stops[1]?.click());
  expect(onStopJob).toHaveBeenCalledWith(2);
});
