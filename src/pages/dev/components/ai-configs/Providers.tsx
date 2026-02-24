import { Button, Header, Input, Selection, TextInput } from "@/components";
import { UseSettingsReturn } from "@/types";
import curl2Json, { ResultJSON } from "@bany/curl-to-json";
import { invoke } from "@tauri-apps/api/core";
import { KeyIcon, TrashIcon } from "lucide-react";
import { useEffect, useState } from "react";

export const Providers = ({
  allAiProviders,
  selectedAIProvider,
  onSetSelectedAIProvider,
  variables,
}: UseSettingsReturn) => {
  const [localSelectedProvider, setLocalSelectedProvider] =
    useState<ResultJSON | null>(null);
  const [openAIKeyInput, setOpenAIKeyInput] = useState("");
  const [openAIKeyConfigured, setOpenAIKeyConfigured] = useState(false);
  const [isOpenAIKeyLoading, setIsOpenAIKeyLoading] = useState(false);
  const isOpenAIProvider = selectedAIProvider?.provider === "openai";

  useEffect(() => {
    if (selectedAIProvider?.provider) {
      const provider = allAiProviders?.find(
        (p) => p?.id === selectedAIProvider?.provider
      );
      if (provider) {
        const json = curl2Json(provider?.curl);
        setLocalSelectedProvider(json as ResultJSON);
      }
    }
  }, [selectedAIProvider?.provider]);

  useEffect(() => {
    if (!isOpenAIProvider) {
      setOpenAIKeyInput("");
      setOpenAIKeyConfigured(false);
      return;
    }

    let isCancelled = false;
    const loadOpenAIKeyStatus = async () => {
      try {
        const status = await invoke<boolean>("get_openai_api_key_status");
        if (!isCancelled) {
          setOpenAIKeyConfigured(status);
        }
      } catch (error) {
        console.error("Failed to get OpenAI API key status:", error);
        if (!isCancelled) {
          setOpenAIKeyConfigured(false);
        }
      }
    };

    loadOpenAIKeyStatus();
    return () => {
      isCancelled = true;
    };
  }, [isOpenAIProvider]);

  const findKeyAndValue = (key: string) => {
    return variables?.find((v) => v?.key === key);
  };

  const getApiKeyValue = () => {
    if (isOpenAIProvider) return openAIKeyInput;
    const apiKeyVar = findKeyAndValue("api_key");
    if (!apiKeyVar || !selectedAIProvider?.variables) return "";
    return selectedAIProvider?.variables?.[apiKeyVar.key] || "";
  };

  const isApiKeyEmpty = () => {
    return !getApiKeyValue().trim();
  };

  const stripOpenAIApiKeyFromLocalState = () => {
    const apiKeyVar = findKeyAndValue("api_key");
    if (!apiKeyVar || !selectedAIProvider?.variables?.[apiKeyVar.key]) return;

    const { [apiKeyVar.key]: _removed, ...rest } = selectedAIProvider.variables;
    onSetSelectedAIProvider({
      ...selectedAIProvider,
      variables: rest,
    });
  };

  const saveOpenAIKey = async () => {
    if (!openAIKeyInput.trim()) return;
    setIsOpenAIKeyLoading(true);
    try {
      await invoke("set_openai_api_key", { key: openAIKeyInput.trim() });
      stripOpenAIApiKeyFromLocalState();
      setOpenAIKeyConfigured(true);
      setOpenAIKeyInput("");
    } catch (error) {
      console.error("Failed to save OpenAI API key:", error);
    } finally {
      setIsOpenAIKeyLoading(false);
    }
  };

  const clearOpenAIKey = async () => {
    setIsOpenAIKeyLoading(true);
    try {
      await invoke("clear_openai_api_key");
      stripOpenAIApiKeyFromLocalState();
      setOpenAIKeyConfigured(false);
      setOpenAIKeyInput("");
    } catch (error) {
      console.error("Failed to clear OpenAI API key:", error);
    } finally {
      setIsOpenAIKeyLoading(false);
    }
  };

  useEffect(() => {
    if (isOpenAIProvider) {
      stripOpenAIApiKeyFromLocalState();
    }
  }, [isOpenAIProvider, selectedAIProvider?.variables]);

  return (
    <div className="space-y-3">
      <div className="space-y-2">
        <Header
          title="Select AI Provider"
          description="Select your preferred AI service provider or custom providers to get started."
        />
        <Selection
          selected={selectedAIProvider?.provider}
          options={allAiProviders?.map((provider) => {
            const json = curl2Json(provider?.curl);
            return {
              label: provider?.isCustom
                ? json?.url || "Custom Provider"
                : provider?.id || "Custom Provider",
              value: provider?.id || "Custom Provider",
              isCustom: provider?.isCustom,
            };
          })}
          placeholder="Choose your AI provider"
          onChange={(value) => {
            onSetSelectedAIProvider({
              provider: value,
              variables: {},
            });
          }}
        />
      </div>

      {localSelectedProvider ? (
        <Header
          title={`Method: ${
            localSelectedProvider?.method || "Invalid"
          }, Endpoint: ${localSelectedProvider?.url || "Invalid"}`}
          description={`If you want to use different url or method, you can always create a custom provider.`}
        />
      ) : null}

      {findKeyAndValue("api_key") ? (
        <div className="space-y-2">
          <Header
            title="API Key"
            description={`Enter your ${
              allAiProviders?.find(
                (p) => p?.id === selectedAIProvider?.provider
              )?.isCustom
                ? "Custom Provider"
                : selectedAIProvider?.provider
            } API key to authenticate and access AI models. ${
              isOpenAIProvider
                ? `Status: ${openAIKeyConfigured ? "Connected" : "Not configured"}`
                : "Your key is stored locally and never shared."
            }`}
          />

          <div className="space-y-2">
            <div className="flex gap-2">
              <Input
                type="password"
                placeholder="**********"
                value={getApiKeyValue()}
                onChange={(value) => {
                  if (isOpenAIProvider) {
                    setOpenAIKeyInput(
                      typeof value === "string" ? value : value.target.value
                    );
                    return;
                  }

                  const apiKeyVar = findKeyAndValue("api_key");
                  if (!apiKeyVar || !selectedAIProvider) return;

                  onSetSelectedAIProvider({
                    ...selectedAIProvider,
                    variables: {
                      ...selectedAIProvider.variables,
                      [apiKeyVar.key]:
                        typeof value === "string" ? value : value.target.value,
                    },
                  });
                }}
                onKeyDown={(e) => {
                  if (isOpenAIProvider) {
                    setOpenAIKeyInput((e.target as HTMLInputElement).value);
                    return;
                  }

                  const apiKeyVar = findKeyAndValue("api_key");
                  if (!apiKeyVar || !selectedAIProvider) return;

                  onSetSelectedAIProvider({
                    ...selectedAIProvider,
                    variables: {
                      ...selectedAIProvider.variables,
                      [apiKeyVar.key]: (e.target as HTMLInputElement).value,
                    },
                  });
                }}
                disabled={false}
                className="flex-1 h-11 border-1 border-input/50 focus:border-primary/50 transition-colors"
              />
              {isOpenAIProvider ? (
                openAIKeyConfigured && isApiKeyEmpty() ? (
                  <Button
                    onClick={clearOpenAIKey}
                    size="icon"
                    variant="destructive"
                    className="shrink-0 h-11 w-11"
                    title="Clear API Key"
                    disabled={isOpenAIKeyLoading}
                  >
                    <TrashIcon className="h-4 w-4" />
                  </Button>
                ) : (
                  <Button
                    onClick={saveOpenAIKey}
                    disabled={isApiKeyEmpty() || isOpenAIKeyLoading}
                    size="icon"
                    className="shrink-0 h-11 w-11"
                    title="Save API Key"
                  >
                    <KeyIcon className="h-4 w-4" />
                  </Button>
                )
              ) : isApiKeyEmpty() ? (
                <Button
                  onClick={() => {
                    const apiKeyVar = findKeyAndValue("api_key");
                    if (!apiKeyVar || !selectedAIProvider || isApiKeyEmpty())
                      return;

                    onSetSelectedAIProvider({
                      ...selectedAIProvider,
                      variables: {
                        ...selectedAIProvider.variables,
                        [apiKeyVar.key]: getApiKeyValue(),
                      },
                    });
                  }}
                  disabled={isApiKeyEmpty()}
                  size="icon"
                  className="shrink-0 h-11 w-11"
                  title="Submit API Key"
                >
                  <KeyIcon className="h-4 w-4" />
                </Button>
              ) : (
                <Button
                  onClick={() => {
                    const apiKeyVar = findKeyAndValue("api_key");
                    if (!apiKeyVar || !selectedAIProvider) return;

                    onSetSelectedAIProvider({
                      ...selectedAIProvider,
                      variables: {
                        ...selectedAIProvider.variables,
                        [apiKeyVar.key]: "",
                      },
                    });
                  }}
                  size="icon"
                  variant="destructive"
                  className="shrink-0 h-11 w-11"
                  title="Remove API Key"
                >
                  <TrashIcon className="h-4 w-4" />
                </Button>
              )}
            </div>
          </div>
        </div>
      ) : null}

      <div className="space-y-4 mt-2">
        {variables
          .filter(
            (variable) => variable.key !== findKeyAndValue("api_key")?.key
          )
          .map((variable) => {
            const getVariableValue = () => {
              if (!variable?.key || !selectedAIProvider?.variables) return "";
              return selectedAIProvider.variables[variable.key] || "";
            };

            return (
              <div className="space-y-1" key={variable?.key}>
                <Header
                  title={variable?.value || ""}
                  description={`add your preferred ${variable?.key?.replace(
                    /_/g,
                    " "
                  )} for ${
                    allAiProviders?.find(
                      (p) => p?.id === selectedAIProvider?.provider
                    )?.isCustom
                      ? "Custom Provider"
                      : selectedAIProvider?.provider
                  }`}
                />
                <TextInput
                  placeholder={`Enter ${
                    allAiProviders?.find(
                      (p) => p?.id === selectedAIProvider?.provider
                    )?.isCustom
                      ? "Custom Provider"
                      : selectedAIProvider?.provider
                  } ${variable?.key?.replace(/_/g, " ") || "value"}`}
                  value={getVariableValue()}
                  onChange={(value) => {
                    if (!variable?.key || !selectedAIProvider) return;

                    onSetSelectedAIProvider({
                      ...selectedAIProvider,
                      variables: {
                        ...selectedAIProvider.variables,
                        [variable.key]: value,
                      },
                    });
                  }}
                />
              </div>
            );
          })}
      </div>
    </div>
  );
};
