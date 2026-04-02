# Краткая сводка проекта (RU)

## Текущее состояние
- Форк Pluely (Tauri + Rust + TypeScript), фокус на Linux/macOS и минимальные изменения.
- OpenAI-путь переведён в backend-first: запросы идут через Tauri/Rust, а не через browser `fetch`.
- Для OpenAI используется `/v1/responses` со streaming; кастомные curl-провайдеры сохранены без изменения архитектуры.
- Настройка `Response Length` теперь влияет не только на prompt, но и на API-параметры ответа (токены/effort).
- Добавлен общий HTTP-клиент `reqwest` в state Tauri для переиспользования keep-alive соединений.
- Screenshot-mode выделен в отдельный stateless AI-path: история не передаётся, изображения становятся основным контекстом, а prompt идёт отдельным `system_prompt`.
- Для main overlay добавлен runtime-переключатель passive/interactive mode на macOS без session persistence.

## Что обновлено в текущем цикле
- Вынесен общий screenshot-pipeline в `src/lib/screenshot/`: константы, builder запроса и shared helper захвата без overlay:
  `src/lib/screenshot/constants.ts:1`, `src/lib/screenshot/buildScreenshotRequest.ts:20`,
  `src/lib/screenshot/captureWithoutOverlay.ts:11`.
- Screenshot capture теперь:
  скрывает окно перед захватом, делает fallback-capture при пустом результате, восстанавливает позицию окна
  и не показывает окно после захвата, если оно было скрыто заранее:
  `src/lib/screenshot/captureWithoutOverlay.ts:19`, `src/lib/screenshot/captureWithoutOverlay.ts:56`,
  `src/lib/screenshot/captureWithoutOverlay.ts:90`, `src/lib/screenshot/captureWithoutOverlay.ts:113`.
- Временные screenshot debug-логи переведены под env-флаг `VITE_SCREENSHOT_DEBUG`:
  `src/lib/utils.ts:8`, `src/pages/app/components/completion/Screenshot.tsx:42`,
  `src/pages/chats/components/ChatScreenshot.tsx:51`.
- Для OpenAI short-mode добавлен отдельный screenshot-бюджет `max_output_tokens=800`,
  чтобы screenshot coding-задачи не обрывались так агрессивно:
  `src-tauri/src/api.rs:19`, `src-tauri/src/api.rs:905`, `src-tauri/src/api.rs:1103`.
- Во frontend system prompt добавлен screenshot-aware override:
  для coding-задач по скриншоту модель должна дать краткое объяснение и затем полный финальный код,
  не подчиняясь ultra-short лимиту 2–4 предложений:
  `src/lib/functions/ai-response.function.ts:19`, `src/lib/functions/ai-response.function.ts:29`.
- Multi-screenshot submission собирается в один AI-запрос и получает внутреннюю подсказку,
  что все screenshots относятся к одной задаче и должны интерпретироваться как единый контекст:
  `src/lib/screenshot/buildScreenshotRequest.ts:20`, `src/lib/screenshot/constants.ts:4`.
- Для OpenAI в multi-turn добавлен компромиссный контекст: последние 3 `user` + 1 последний `assistant`.
  Роли в `input` для `/v1/responses`: `user -> input_text`, `assistant -> output_text`:
  `src-tauri/src/api.rs:15`, `src-tauri/src/api.rs:705`, `src-tauri/src/api.rs:899`, `src-tauri/src/api.rs:996`.
- Для `short` увеличен лимит среза ответа до `max_output_tokens=200`:
  `src-tauri/src/api.rs:18`, `src-tauri/src/api.rs:1021`.
- Backend-команда `chat_stream_response` расширена для OpenAI `/v1/responses`, включая mapping `instructions/input` и метрики:
  `src-tauri/src/api.rs:820`, `src-tauri/src/api.rs:1027`, `src-tauri/src/api.rs:1287`.
- OpenAI API key перенесён в backend-хранилище: команды `set/get_status/clear` + регистрация в invoke handler:
  `src-tauri/src/api.rs:215`, `src-tauri/src/api.rs:234`, `src-tauri/src/lib.rs:112`.
- OpenAI больше не блокируется лицензионной проверкой в direct backend path при наличии пользовательского ключа:
  `src-tauri/src/api.rs:834`, `src-tauri/src/api.rs:875`.
- UI настроек провайдера: статус `Connected/Not configured`, сохранение/очистка ключа через invoke:
  `src/pages/dev/components/ai-configs/Providers.tsx:43`, `src/pages/dev/components/ai-configs/Providers.tsx:172`.
- Ключ OpenAI убран из `localStorage` для выбранного провайдера:
  `src/contexts/app.context.tsx:245`, `src/contexts/app.context.tsx:510`.
- `Response Length` прокинут из frontend в backend и влияет на параметры OpenAI:
  `src/lib/functions/ai-response.function.ts:211`, `src/lib/functions/ai-response.function.ts:122`,
  `src-tauri/src/api.rs:824`, `src-tauri/src/api.rs:1020`.
- Для `auto` установлен фиксированный лимит `max_output_tokens = 800`:
  `src-tauri/src/api.rs:20`, `src-tauri/src/api.rs:1023`.
- Добавлено измерение TTFB первого чанка в backend-логах:
  `src-tauri/src/api.rs:1186`, `src-tauri/src/api.rs:1222`, `src-tauri/src/api.rs:1287`.
- Введён единый shared `reqwest::Client` (pooling/keep-alive) и использование его в `chat_stream_response`:
  `src-tauri/src/lib.rs:37`, `src-tauri/src/lib.rs:47`, `src-tauri/src/api.rs:817`, `src-tauri/src/api.rs:1115`.
- Добавлен фоновый one-shot warm-up OpenAI при старте (не блокирует UI):
  `src-tauri/src/api.rs:113`, `src-tauri/src/lib.rs:213`.
- Временно заглушены shortcut-логи через локальный макрос:
  `src-tauri/src/shortcuts.rs:15`, `src-tauri/src/lib.rs:169`.
- Для main window добавлен backend state passive mode + команды `set_main_window_passive` / `get_main_window_passive`:
  `src-tauri/src/window.rs:13`, `src-tauri/src/window.rs:159`, `src-tauri/src/window.rs:187`,
  `src-tauri/src/lib.rs:58`, `src-tauri/src/lib.rs:89`.
- На macOS passive mode теперь можно переключать в runtime:
  `passive=true` делает окно non-focusable и click-through, `passive=false` возвращает обычную интерактивность:
  `src-tauri/src/window.rs:61`.
- В существующую shortcut-систему добавлено действие `toggle_passive_mode` с дефолтом `Cmd/Ctrl+Shift+P`:
  `src/config/shortcuts.ts:85`.
- Frontend синхронизирует passive mode с backend через runtime state и событие `passive-mode-changed`;
  состояние сессионное, без `localStorage`:
  `src/contexts/app.context.tsx:150`, `src/contexts/app.context.tsx:370`, `src/contexts/app.context.tsx:713`.
- В overlay UI добавлен компактный индикатор текущего режима `Passive / Interactive`:
  `src/pages/app/index.tsx:78`.

## AI Pipeline

### Screenshot Flow
- Screenshot-запросы работают в stateless режиме: предыдущая история не передаётся в OpenAI-path, чтобы прошлые screenshot Q&A не влияли на новый анализ.
- В screenshot-mode используется отдельный `system_prompt`, а изображения передаются как основной контекст текущего запроса.
- Для manual mode несколько screenshots объединяются в один запрос; при количестве больше одного добавляется внутренний hint, что это одна задача.
- Shared capture helper скрывает окно перед захватом, делает fallback при пустом hidden-capture и восстанавливает окно только если оно было видно до старта:
  `src/lib/screenshot/captureWithoutOverlay.ts:11`.

### Response Control
- `short` по умолчанию: `max_output_tokens = 200`.
- `short + code_intent`: `max_output_tokens = 450`.
- `short + screenshotMode`: `max_output_tokens = 800`.
- `medium`: `max_output_tokens = 600`.
- `auto`: `max_output_tokens = 800`.
- Для OpenAI `/v1/responses` эти ограничения применяются на backend и логируются вместе с `response_length`, `screenshot_mode` и `code_intent`:
  `src-tauri/src/api.rs:1103`, `src-tauri/src/api.rs:1184`.

### Prompt System
- Базовый screenshot prompt берётся из настроек screenshot mode; если он пуст, используется дефолтный prompt:
  `src/lib/screenshot/buildScreenshotRequest.ts:39`, `src/lib/screenshot/constants.ts:1`.
- Текущая screenshot prompt v2-логика в коде состоит из stateless screenshot-mode, multi-screenshot context hint и отдельного short-mode coding override.
- Для short-mode screenshot coding-задач автоматически добавляется отдельный prompt-override:
  модель должна сначала кратко объяснить подход, затем дать полный финальный код, добавить короткие полезные комментарии и не возвращать partial code:
  `src/lib/functions/ai-response.function.ts:19`.
- Текущая prompt-логика уже фиксирует stateless screenshot-mode и multi-screenshot context hint.
- Более строгие правила уровня «игнорировать UI-шум», «трактовать комментарии как часть спецификации» и interview-style fallback для async/event loop задач пока нужно валидировать отдельно; они не зафиксированы в коде как гарантированный общий prompt-layer.

## Известные ограничения/заметки
- Основная логика screenshot capture централизована, но требует ручной QA на разных Linux WM/композиторах.
- Fallback-capture при скрытии overlay снижает риск пустого кадра, но может зависеть от таймингов окружения.
- Нужна ручная валидация latency/output_tokens по профилям `short/medium/auto` на реальных промптах.
- Текущее secure storage реализовано в рамках существующего локального механизма проекта; отдельной миграции на OS keychain пока нет.
- Для OpenAI добавлены диагностические backend-логи по длине ответа, фильтрации контекста и первому чанку.
- Из-за временного mute shortcut-логов снижена диагностируемость проблем hotkeys.
- Passive mode пока ориентирован на macOS; на других платформах backend оставляет поведение безопасным no-op.
- По результатам ручного тестирования на macOS приложение падает при переключении passive/interactive mode через горячую клавишу; баг не исправлен и требует отдельного разбора.

## Ограничения
- Качество screenshot-анализа зависит от читаемости изображения, масштаба, контраста и плотности текста.
- Возможны OCR-like ошибки: модель может неточно прочитать отдельные токены, имена переменных или мелкие фрагменты интерфейса.
- Без дополнительного контекста пользователя модель может неверно интерпретировать intent задачи, особенно если вопрос явно не сформулирован.
- Edge-case задачи по `async`/event loop/microtasks/macrotasks требуют отдельной ручной проверки; интерпретация таких screenshots может быть неидеальной.

## TODO/вопросы
- Проверить screenshot-flow в сценариях visible/hidden/manual/auto на Linux и macOS (TODO).
- Подтвердить, что screenshot coding tasks в `short` теперь дают полный код без обрывов.
- Отдельно валидировать richer screenshot prompt-policy для interview-style задач; пока она не зафиксирована как универсальный hardcoded слой.
- Подтвердить целевые SLA по latency и токенам для `short/medium/auto` (TODO: зафиксировать числа).
- Решить, нужен ли настраиваемый лимит для `auto` вместо фиксированного `800`.
- Проверить отдельным сценарием OpenAI multi-turn с изображениями после обновлённой фильтрации истории.
- Вернуть shortcut-логи под debug/env-флаг после стабилизации hotkeys.
- Проверить и исправить crash на macOS при переключении passive/interactive mode через shortcut.
