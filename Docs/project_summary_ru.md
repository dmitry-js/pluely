# Краткая сводка проекта (RU)

## Текущее состояние
- Форк Pluely (Tauri + Rust + TypeScript), фокус на Linux/macOS и минимальные изменения.
- OpenAI-путь переведён в backend-first: запросы идут через Tauri/Rust, а не через browser `fetch`.
- Для OpenAI используется `/v1/responses` со streaming; кастомные curl-провайдеры сохранены без изменения архитектуры.
- Настройка `Response Length` теперь влияет не только на prompt, но и на API-параметры ответа (токены/effort).
- Добавлен общий HTTP-клиент `reqwest` в state Tauri для переиспользования keep-alive соединений.
- Screenshot-mode выделен в отдельный stateless AI-path: история не передаётся, изображения становятся основным контекстом, а prompt идёт отдельным `system_prompt`.
- Runtime toggle passive/interactive mode на macOS временно заморожен: приоритет смещён в сторону стабильности окна и UX-консистентности.

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
- Для main window сохранён backend state passive mode + команды `set_main_window_passive` / `get_main_window_passive`,
  но на macOS runtime toggle временно переведён в безопасный no-op до panel-safe реализации:
  `src-tauri/src/window.rs:13`, `src-tauri/src/window.rs:61`, `src-tauri/src/window.rs:155`.
- В существующую shortcut-систему сохранено действие `toggle_passive_mode` с дефолтом `Cmd/Ctrl+Shift+P`,
  но на macOS оно явно помечено как временно недоступное в UI настроек:
  `src/config/shortcuts.ts:85`, `src/pages/shortcuts/components/shortcuts/ShortcutManager.tsx:211`.
- Frontend guard для macOS больше не вызывает backend passive-mode toggle и не меняет UI state,
  чтобы не создавать ложного поведения:
  `src/contexts/app.context.tsx:713`.
- Из overlay UI на macOS убран индикатор passive mode, чтобы не показывать нерабочий runtime-control:
  `src/pages/app/index.tsx:21`.
- Crash native opacity на macOS устранён:
  runtime update больше не делает повторный `to_panel()`, а переиспользует уже созданный panel через
  `get_webview_panel(...)`; это убирает небезопасный runtime re-panelization:
  `src-tauri/src/window.rs:209`, `src-tauri/src/lib.rs:241`.
- Для transparency slider на macOS применён stability-first режим:
  во время drag обновляется локальное значение UI, а native opacity коммитится только по завершению drag:
  `src/pages/settings/components/Theme.tsx:108`.
- Keyboard-only UX для overlay улучшен:
  `Esc` больше не скрывает response panel и ведёт себя как soft-cancel/blur, добавлен shortcut `toggle_response_panel`,
  а для длинных ответов появились keyboard shortcuts на scroll response viewport:
  `src/hooks/useCompletion.ts:1041`, `src/config/shortcuts.ts:31`, `src/config/shortcuts.ts:51`.
- Shortcut `Refocus Input Box` снова работает, включая возврат фокуса после потери активного приложения:
  backend сначала фокусирует окно, затем frontend фокусирует input:
  `src-tauri/src/shortcuts.rs:768`, `src/hooks/useGlobalShortcuts.ts:23`.
- Текущий фокус цикла: стабильность window lifecycle, opacity и предсказуемость overlay UX вместо расширения feature-surface.

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
- Runtime passive mode toggle на macOS временно отключён; shortcut и overlay UI приведены к честному degraded state без ложных ожиданий.
- Native opacity на macOS теперь стабилен в обычном runtime path; следующая зона риска — window/panel lifecycle при show/hide и других hotkey flow.
- На macOS остаётся follow-up на более мягкий `refocus input` behavior без ощущения жёсткого app-switch; это polish-задача, не текущий блокер.

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
- Вернуть macOS passive mode только после panel-safe реализации и отдельного smoke-test набора.
- Дожать macOS window lifecycle после фикса opacity и зафиксировать критерии приёмки для hide/show и panel state.
- Отдельно отполировать macOS refocus/input UX, чтобы возврат фокуса ощущался мягче при keyboard-first overlay.
