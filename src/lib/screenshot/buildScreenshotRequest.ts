import type { ScreenshotConfig } from "@/types/settings";
import {
  DEFAULT_SCREENSHOT_PROMPT,
  DEFAULT_SCREENSHOT_USER_MESSAGE,
  MULTI_SCREENSHOT_CONTEXT_HINT,
} from "./constants";

type ScreenshotAttachment = {
  type: string;
  base64: string;
};

export type ScreenshotRequestPayload = {
  userMessage: string;
  systemPrompt: string;
  imagesBase64: string[];
  screenshotMode: true;
};

export const buildScreenshotRequest = ({
  input,
  attachedFiles,
  screenshotConfiguration,
}: {
  input: string;
  attachedFiles: ScreenshotAttachment[];
  screenshotConfiguration: ScreenshotConfig;
}): ScreenshotRequestPayload | null => {
  const attachedImages = attachedFiles.filter((file) =>
    file.type.startsWith("image/")
  );

  if (attachedImages.length === 0) {
    return null;
  }

  const userMessage = input.trim() || DEFAULT_SCREENSHOT_USER_MESSAGE;
  const basePrompt =
    screenshotConfiguration.autoPrompt?.trim() || DEFAULT_SCREENSHOT_PROMPT;
  const systemPrompt =
    attachedImages.length > 1
      ? `${basePrompt}\n\n${MULTI_SCREENSHOT_CONTEXT_HINT}`
      : basePrompt;

  return {
    userMessage,
    systemPrompt,
    imagesBase64: attachedImages.map((file) => file.base64),
    screenshotMode: true,
  };
};
