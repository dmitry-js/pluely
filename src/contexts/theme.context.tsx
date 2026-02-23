import { createContext, useContext, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { STORAGE_KEYS } from "@/config/";

type Theme = "dark" | "light" | "system";

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
};

type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  transparency: number;
  onSetTransparency: (transparency: number) => void;
};

const initialState: ThemeProviderState = {
  theme: "system",
  setTheme: () => null,
  transparency: 10,
  onSetTransparency: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

export function ThemeProvider({
  children,
  defaultTheme = "system",
  storageKey = STORAGE_KEYS.THEME,
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme
  );
  const [transparency, setTransparency] = useState<number>(() => {
    const stored = localStorage.getItem(STORAGE_KEYS.TRANSPARENCY);
    return stored ? parseInt(stored, 10) : 10;
  });

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const isSystemThemeDark = mediaQuery.matches;

  useEffect(() => {
    const handleStorageChange = (e: StorageEvent) => {
      if (e.key === STORAGE_KEYS.TRANSPARENCY && e.newValue) {
        setTransparency(parseInt(e.newValue, 10));
      }
      if (e.key === storageKey && e.newValue) {
        setTheme(e.newValue as Theme);
      }
    };

    window.addEventListener("storage", handleStorageChange);
    return () => window.removeEventListener("storage", handleStorageChange);
  }, [storageKey]);

  useEffect(() => {
    const root = window.document.documentElement;

    const applyTheme = (currentTheme: Theme) => {
      root.classList.remove("light", "dark");

      if (currentTheme === "system") {
        const systemTheme = mediaQuery.matches ? "dark" : "light";
        root.classList.add(systemTheme);
      } else {
        root.classList.add(currentTheme);
      }
    };

    const updateTheme = () => {
      if (theme === "system") {
        applyTheme("system");
      }
    };

    applyTheme(theme);

    if (theme === "system") {
      mediaQuery.addEventListener("change", updateTheme);
    }

    return () => {
      if (theme === "system") {
        mediaQuery.removeEventListener("change", updateTheme);
      }
    };
  }, [theme]);

  // Apply native window opacity from transparency setting
  useEffect(() => {
    const root = window.document.documentElement;
    root.style.setProperty("--opacity", "1");
    root.style.setProperty("--backdrop-blur", "none");

    const normalized = Math.max(0, Math.min(100, transparency));
    const opacity = (100 - normalized) / 100;

    const timeoutId = window.setTimeout(() => {
      if (!("__TAURI_INTERNALS__" in window)) {
        return;
      }

      invoke("set_window_opacity", { opacity }).catch((error) => {
        console.error("Failed to set window opacity:", error);
      });
    }, 75);

    return () => window.clearTimeout(timeoutId);
  }, [transparency]);

  const onSetTransparency = (transparency: number) => {
    localStorage.setItem(STORAGE_KEYS.TRANSPARENCY, transparency.toString());
    setTransparency(transparency);
  };

  const value = {
    theme,
    setTheme: (newTheme: Theme) => {
      localStorage.setItem(storageKey, newTheme);
      setTheme(newTheme);
    },
    isSystemThemeDark,
    transparency,
    onSetTransparency,
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider");

  return context;
};
