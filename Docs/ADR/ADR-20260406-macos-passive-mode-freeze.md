# ADR: Временная заморозка runtime passive mode на macOS

## Контекст
После первичной интеграции `toggle_passive_mode` выяснилось, что runtime-переключение passive/interactive режима на macOS конфликтует с текущим lifecycle panelized overlay (`NSPanel`) и приводит к нестабильному поведению в hotkey flow. Попытка чинить toggle локально расширила зону регрессий на hide/screenshot/window сценарии. На текущем этапе важнее сохранить честный UX и стабильность, чем удерживать частично рабочую функцию.

## Решение
1. Временно отключить фактический runtime toggle passive mode на macOS.
2. Сохранить shortcut `toggle_passive_mode` в настройках, но показывать его как временно недоступный на macOS.
3. Убрать passive mode control/indicator из overlay UI на macOS, чтобы не демонстрировать нерабочее состояние.
4. Вернуться к реализации только после появления panel-safe подхода для passive mode и отдельной проверки opacity/window behavior.

## Альтернативы (кратко)
- Попытаться сразу дочинить toggle через `NSPanel`: отклонено в этой итерации, т.к. растёт риск новых регрессий в lifecycle окна.
- Полный откат passive mode state/shortcut из проекта: отклонено, т.к. состояние и конфигурационная точка ещё пригодятся после стабилизации.
- Оставить UI и shortcut как есть: отклонено, т.к. это создаёт ложное ожидание рабочей функции и ухудшает UX-консистентность.

## Последствия
- UX на macOS стал честнее: пользователь не видит вводящий в заблуждение control в overlay.
- Функциональность временно урезана: runtime passive mode toggle недоступен до отдельного исправления.
- Приоритет разработки смещён в сторону стабильности окна, opacity и реальных hotkey-сценариев.

## Ссылки/файлы
- `src-tauri/src/window.rs:61`
- `src-tauri/src/window.rs:155`
- `src/contexts/app.context.tsx:713`
- `src/pages/shortcuts/components/shortcuts/ShortcutManager.tsx:211`
- `src/pages/app/index.tsx:21`
- `Docs/ADR/ADR-20260403-key-decisions.md`
