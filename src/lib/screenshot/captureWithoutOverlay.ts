import { invoke } from "@tauri-apps/api/core";
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { logScreenshotDebug } from "@/lib/utils";
import {
  SCREENSHOT_CAPTURE_HIDE_DELAY_MS,
  SCREENSHOT_FALLBACK_CAPTURE_DELAY_MS,
  SCREENSHOT_RESTORE_POSITION_DELAY_MS,
} from "./constants";

export const captureWithoutOverlay = async (): Promise<string | null> => {
  const win = getCurrentWindow();
  let shouldRestoreWindow = false;
  let savedPosition: { x: number; y: number } | null = null;
  let wasVisibleBeforeCapture: boolean | null = null;
  const canUseVisibleFallback = () => wasVisibleBeforeCapture !== false;

  logScreenshotDebug("[screenshot] capture start");

  try {
    wasVisibleBeforeCapture = await win.isVisible();
    logScreenshotDebug("[screenshot] visible before capture", {
      wasVisibleBeforeCapture,
    });
  } catch (error) {
    console.warn(
      "Failed to read window visibility before screenshot capture:",
      error
    );
  }

  try {
    const position = await win.outerPosition();
    savedPosition = { x: position.x, y: position.y };
    logScreenshotDebug("[screenshot] saved position before hide", savedPosition);
  } catch (error) {
    console.warn(
      "Failed to read window position before screenshot capture:",
      error
    );
  }

  try {
    await win.hide();
    shouldRestoreWindow = wasVisibleBeforeCapture ?? true;
  } catch (error) {
    console.warn("Failed to hide window before screenshot capture:", error);
  }

  try {
    if (shouldRestoreWindow) {
      await new Promise((resolve) =>
        setTimeout(resolve, SCREENSHOT_CAPTURE_HIDE_DELAY_MS)
      );
    }

    let base64 = await invoke<string>("capture_to_base64");
    if (!base64 || typeof base64 !== "string" || base64.length === 0) {
      if (canUseVisibleFallback()) {
        console.warn(
          "[screenshot] hidden capture failed, retrying without overlay hide"
        );

        try {
          await win.show();
        } catch {}

        await new Promise((resolve) =>
          setTimeout(resolve, SCREENSHOT_FALLBACK_CAPTURE_DELAY_MS)
        );
        base64 = await invoke<string>("capture_to_base64");
      }
    }

    if (!base64 || typeof base64 !== "string" || base64.length === 0) {
      console.warn("[screenshot] capture failed: empty image data");
      return null;
    }

    logScreenshotDebug("[screenshot] capture success", base64.length);
    return base64;
  } finally {
    if (shouldRestoreWindow) {
      try {
        await win.show();
      } catch (error) {
        console.warn(
          "Failed to restore window visibility after capture:",
          error
        );
      }

      if (savedPosition) {
        await new Promise((resolve) =>
          setTimeout(resolve, SCREENSHOT_RESTORE_POSITION_DELAY_MS)
        );

        try {
          await win.setPosition(
            new PhysicalPosition(savedPosition.x, savedPosition.y)
          );
          logScreenshotDebug(
            "[screenshot] restored position after show",
            savedPosition
          );
        } catch (error) {
          console.warn("Failed to restore window position after capture:", error);
        }
      }
    } else if (wasVisibleBeforeCapture === false) {
      logScreenshotDebug(
        "[screenshot] restore skipped because window was already hidden before capture"
      );
    }
  }
};
