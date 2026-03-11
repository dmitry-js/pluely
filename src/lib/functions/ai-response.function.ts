import {
  buildDynamicMessages,
  deepVariableReplacer,
  extractVariables,
  getByPath,
  getStreamingContent,
} from "./common.function";
import { Message, TYPE_PROVIDER } from "@/types";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import curl2Json from "@bany/curl-to-json";
import { shouldUsePluelyAPI } from "./pluely.api";
import { getResponseSettings, RESPONSE_LENGTHS, LANGUAGES } from "@/lib";
import { MARKDOWN_FORMATTING_INSTRUCTIONS } from "@/config/constants";

type ResponseLengthId = "short" | "medium" | "auto";

function normalizeResponseLength(value?: string): ResponseLengthId {
  if (value === "short" || value === "medium" || value === "auto") {
    return value;
  }
  return "short";
}

function buildEnhancedSystemPrompt(baseSystemPrompt?: string): string {
  const responseSettings = getResponseSettings();
  const prompts: string[] = [];

  if (baseSystemPrompt) {
    prompts.push(baseSystemPrompt);
  }

  const lengthOption = RESPONSE_LENGTHS.find(
    (l) => l.id === responseSettings.responseLength
  );
  if (lengthOption?.prompt?.trim()) {
    prompts.push(lengthOption.prompt);
  }

  const languageOption = LANGUAGES.find(
    (l) => l.id === responseSettings.language
  );
  if (languageOption?.prompt?.trim()) {
    prompts.push(languageOption.prompt);
  }

  // Add markdown formatting instructions
  prompts.push(MARKDOWN_FORMATTING_INSTRUCTIONS);

  return prompts.join(" ");
}

// Backend AI streaming function
async function* fetchBackendAIResponse(params: {
  systemPrompt?: string;
  userMessage: string;
  imagesBase64?: string[];
  history?: Message[];
  screenshotMode?: boolean;
  provider?: string;
  model?: string;
  responseLength?: ResponseLengthId;
  signal?: AbortSignal;
}): AsyncIterable<string> {
  try {
    const {
      systemPrompt,
      userMessage,
      imagesBase64 = [],
      history = [],
      screenshotMode = false,
      provider,
      model,
      responseLength,
      signal,
    } = params;

    // Check if already aborted before starting
    if (signal?.aborted) {
      return;
    }

    // Convert history to the expected format
    let historyString: string | undefined;
    if (!screenshotMode && history.length > 0) {
      // Create a copy before reversing to avoid mutating the original array
      const formattedHistory = [...history].reverse().map((msg) => ({
        role: msg.role,
        content: [{ type: "text", text: msg.content }],
      }));
      historyString = JSON.stringify(formattedHistory);
    }

    // Handle images - can be string or array
    let imageBase64: any = undefined;
    if (imagesBase64.length > 0) {
      imageBase64 = imagesBase64.length === 1 ? imagesBase64[0] : imagesBase64;
    }

    const isDev = import.meta.env.DEV;
    const invokeStartedAt = Date.now();
    let firstChunkAt: number | null = null;
    let chunkCount = 0;

    if (isDev) {
      console.debug(
        `[stream][backend] invoke start at ${new Date(
          invokeStartedAt
        ).toISOString()}`
      );
      if (screenshotMode) {
        console.debug(
          "[stream][backend] screenshot_mode=true history_included=false system_prompt_source=screenshot_mode"
        );
      }
    }

    // Set up streaming event listeners BEFORE invoke to avoid missing early chunks.
    const streamQueue: string[] = [];
    let streamDone = false;
    let invokeSettled = false;
    let invokeError: unknown = null;
    let pendingResolve: (() => void) | null = null;

    const notify = () => {
      if (pendingResolve) {
        const resolve = pendingResolve;
        pendingResolve = null;
        resolve();
      }
    };

    let unlistenChunk: () => void = () => {};
    let unlistenComplete: () => void = () => {};
    let unlistenDone: () => void = () => {};
    const onAbort = () => notify();

    try {
      unlistenChunk = await listen("chat_stream_chunk", (event) => {
        const chunk = event.payload as string;
        streamQueue.push(chunk);
        chunkCount += 1;

        if (firstChunkAt === null) {
          firstChunkAt = Date.now();
          if (isDev) {
            console.debug(
              `[stream][backend] first chunk at ${new Date(
                firstChunkAt
              ).toISOString()} (+${firstChunkAt - invokeStartedAt}ms)`
            );
          }
        }

        notify();
      });

      unlistenComplete = await listen("chat_stream_complete", () => {
        streamDone = true;
        notify();
      });

      // Optional compatibility event.
      unlistenDone = await listen("chat_stream_done", () => {
        streamDone = true;
        notify();
      });

      // Check if aborted before starting invoke
      if (signal?.aborted) {
        return;
      }

      signal?.addEventListener("abort", onAbort);

      // Start invoke in parallel and drain queue immediately.
      const invokePromise = invoke("chat_stream_response", {
        userMessage,
        systemPrompt,
        imageBase64,
        history: historyString,
        provider,
        model,
        responseLength,
      })
        .catch((error) => {
          invokeError = error;
        })
        .finally(() => {
          invokeSettled = true;
          streamDone = true;
          notify();
        });

      // Yield chunks as soon as they arrive.
      while (true) {
        if (signal?.aborted) {
          return;
        }

        while (streamQueue.length > 0) {
          const nextChunk = streamQueue.shift();
          if (nextChunk) {
            yield nextChunk;
          }
        }

        if (invokeError) {
          throw invokeError;
        }

        if (invokeSettled && streamDone && streamQueue.length === 0) {
          break;
        }

        await new Promise<void>((resolve) => {
          pendingResolve = resolve;
        });
      }

      await invokePromise;

      if (isDev) {
        const doneAt = Date.now();
        console.debug(
          `[stream][backend] stream done at ${new Date(
            doneAt
          ).toISOString()} (chunks=${chunkCount})`
        );
      }
    } finally {
      signal?.removeEventListener("abort", onAbort);
      if (pendingResolve) {
        pendingResolve();
        pendingResolve = null;
      }
      unlistenChunk();
      unlistenComplete();
      unlistenDone();
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    yield `Backend AI Error: ${errorMessage}`;
  }
}

export async function* fetchAIResponse(params: {
  provider: TYPE_PROVIDER | undefined;
  selectedProvider: {
    provider: string;
    variables: Record<string, string>;
  };
  systemPrompt?: string;
  history?: Message[];
  userMessage: string;
  imagesBase64?: string[];
  screenshotMode?: boolean;
  signal?: AbortSignal;
}): AsyncIterable<string> {
  try {
    const {
      provider,
      selectedProvider,
      systemPrompt,
      history = [],
      userMessage,
      imagesBase64 = [],
      screenshotMode = false,
      signal,
    } = params;

    // Check if already aborted
    if (signal?.aborted) {
      return;
    }

    const enhancedSystemPrompt = buildEnhancedSystemPrompt(systemPrompt);
    const responseLength = normalizeResponseLength(
      getResponseSettings().responseLength
    );

    // Check if we should use Pluely API instead
    const usePluelyAPI = await shouldUsePluelyAPI();
    if (usePluelyAPI) {
      yield* fetchBackendAIResponse({
        systemPrompt: enhancedSystemPrompt,
        userMessage,
        imagesBase64,
        history,
        screenshotMode,
        responseLength,
        signal,
      });
      return;
    }
    if (!provider) {
      throw new Error(`Provider not provided`);
    }
    if (!selectedProvider) {
      throw new Error(`Selected provider not provided`);
    }

    const isOpenAIProvider =
      provider?.id === "openai" || selectedProvider?.provider === "openai";
    if (isOpenAIProvider) {
      const selectedModel =
        selectedProvider?.variables?.model ||
        selectedProvider?.variables?.MODEL ||
        "";

      yield* fetchBackendAIResponse({
        systemPrompt: enhancedSystemPrompt,
        userMessage,
        imagesBase64,
        history,
        screenshotMode,
        provider: "openai",
        model: selectedModel,
        responseLength,
        signal,
      });
      return;
    }

    let curlJson;
    try {
      curlJson = curl2Json(provider.curl);
    } catch (error) {
      throw new Error(
        `Failed to parse curl: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }

    const extractedVariables = extractVariables(provider.curl);
    const requiredVars = extractedVariables.filter(
      ({ key }) => key !== "SYSTEM_PROMPT" && key !== "TEXT" && key !== "IMAGE"
    );
    for (const { key } of requiredVars) {
      if (
        !selectedProvider.variables?.[key] ||
        selectedProvider.variables[key].trim() === ""
      ) {
        throw new Error(
          `Missing required variable: ${key}. Please configure it in settings.`
        );
      }
    }

    if (!userMessage) {
      throw new Error("User message is required");
    }
    if (imagesBase64.length > 0 && !provider.curl.includes("{{IMAGE}}")) {
      throw new Error(
        `Provider ${provider?.id ?? "unknown"} does not support image input`
      );
    }

    let bodyObj: any = curlJson.data
      ? JSON.parse(JSON.stringify(curlJson.data))
      : {};
    const messagesKey = Object.keys(bodyObj).find((key) =>
      ["messages", "contents", "conversation", "history"].includes(key)
    );

    if (messagesKey && Array.isArray(bodyObj[messagesKey])) {
      const finalMessages = buildDynamicMessages(
        bodyObj[messagesKey],
        history,
        userMessage,
        imagesBase64
      );
      bodyObj[messagesKey] = finalMessages;
    }

    const allVariables = {
      ...Object.fromEntries(
        Object.entries(selectedProvider.variables).map(([key, value]) => [
          key.toUpperCase(),
          value,
        ])
      ),
      SYSTEM_PROMPT: enhancedSystemPrompt || "",
    };

    bodyObj = deepVariableReplacer(bodyObj, allVariables);
    let url = deepVariableReplacer(curlJson.url || "", allVariables);

    const headers = deepVariableReplacer(curlJson.header || {}, allVariables);
    headers["Content-Type"] = "application/json";

    if (provider?.streaming) {
      if (typeof bodyObj === "object" && bodyObj !== null) {
        const streamKey = Object.keys(bodyObj).find(
          (k) => k.toLowerCase() === "stream"
        );
        if (streamKey) {
          bodyObj[streamKey] = true;
        } else {
          bodyObj.stream = true;
        }
      }
    }

    const fetchFunction = url?.includes("http") ? fetch : tauriFetch;

    let response;
    try {
      response = await fetchFunction(url, {
        method: curlJson.method || "POST",
        headers,
        body: curlJson.method === "GET" ? undefined : JSON.stringify(bodyObj),
        signal,
      });
    } catch (fetchError) {
      // Check if aborted
      if (
        signal?.aborted ||
        (fetchError instanceof Error && fetchError.name === "AbortError")
      ) {
        return; // Silently return on abort
      }
      yield `Network error during API request: ${
        fetchError instanceof Error ? fetchError.message : "Unknown error"
      }`;
      return;
    }

    if (!response.ok) {
      let errorText = "";
      try {
        errorText = await response.text();
      } catch {}
      yield `API request failed: ${response.status} ${response.statusText}${
        errorText ? ` - ${errorText}` : ""
      }`;
      return;
    }

    if (!provider?.streaming) {
      let json;
      try {
        json = await response.json();
      } catch (parseError) {
        yield `Failed to parse non-streaming response: ${
          parseError instanceof Error ? parseError.message : "Unknown error"
        }`;
        return;
      }
      const content =
        getByPath(json, provider?.responseContentPath || "") || "";
      yield content;
      return;
    }

    if (!response.body) {
      yield "Streaming not supported or response body missing";
      return;
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
      // Check if aborted
      if (signal?.aborted) {
        reader.cancel();
        return;
      }

      let readResult;
      try {
        readResult = await reader.read();
      } catch (readError) {
        // Check if aborted
        if (
          signal?.aborted ||
          (readError instanceof Error && readError.name === "AbortError")
        ) {
          return; // Silently return on abort
        }
        yield `Error reading stream: ${
          readError instanceof Error ? readError.message : "Unknown error"
        }`;
        return;
      }
      const { done, value } = readResult;
      if (done) break;

      // Check if aborted before processing
      if (signal?.aborted) {
        reader.cancel();
        return;
      }

      buffer += decoder.decode(value, { stream: true });

      const lines = buffer.split("\n");
      buffer = lines.pop() || "";
      for (const line of lines) {
        if (line.startsWith("data:")) {
          const trimmed = line.substring(5).trim();
          if (!trimmed || trimmed === "[DONE]") continue;
          try {
            const parsed = JSON.parse(trimmed);
            const delta = getStreamingContent(
              parsed,
              provider?.responseContentPath || ""
            );
            if (delta) {
              yield delta;
            }
          } catch (e) {
            // Ignore parsing errors for partial JSON chunks
          }
        }
      }
    }
  } catch (error) {
    throw new Error(
      `Error in fetchAIResponse: ${
        error instanceof Error ? error.message : "Unknown error"
      }`
    );
  }
}
