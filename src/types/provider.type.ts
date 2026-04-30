export interface TYPE_PROVIDER {
  id?: string;
  streaming?: boolean;
  responseContentPath?: string;
  isCustom?: boolean;
  defaultVariables?: Record<string, string>;
  curl: string;
}
