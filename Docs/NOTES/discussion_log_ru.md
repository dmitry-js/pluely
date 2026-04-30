## 2026-02-24
- Ключевые пункты: OpenAI переведён на backend-only путь через `chat_stream_response`; browser `fetch` для OpenAI исключён.
- Ключевые пункты: добавлены backend-команды хранения OpenAI API key и статус/очистка в UI настроек.
- Ключевые пункты: OpenAI-путь развязан с лицензией при наличии пользовательского ключа.
- Ключевые пункты: для OpenAI история урезана до user-only и последних 3 сообщений для ускорения.
- Ключевые пункты: `Response Length` теперь влияет на `max_output_tokens` и `reasoning.effort` (`auto` ограничен 800).
- Риски/вопросы: проверить, что лимит `auto=800` не ухудшает качество на сложных запросах.
- Риски/вопросы: зафиксировать целевые latency/token-метрики для short/medium/auto (TODO).
- Следующие шаги: прогнать ручной A/B тест скорости и длины ответа по трём режимам.
- Следующие шаги: проверить multi-turn сценарии с OpenAI и отсутствие ошибок `input_text`.
- Следующие шаги: подтвердить, что кастомные curl-провайдеры не затронуты.

## 2026-03-02
- Ключевые пункты: для OpenAI-контекста внедрён баланс «3 последних user + 1 последний assistant», чтобы снизить повторы между ходами (`src-tauri/src/api.rs:899`).
- Ключевые пункты: добавлен корректный mapping `assistant -> output_text` для `/v1/responses` (`src-tauri/src/api.rs:705`).
- Ключевые пункты: `short` увеличен до `200` токенов для уменьшения обрывов ответа (`src-tauri/src/api.rs:18`).
- Ключевые пункты: внедрён shared `reqwest::Client` + лог TTFB первого чанка (`src-tauri/src/lib.rs:37`, `src-tauri/src/api.rs:1222`).
- Ключевые пункты: добавлен неблокирующий warm-up OpenAI на старте, one-shot на запуск (`src-tauri/src/api.rs:113`, `src-tauri/src/lib.rs:213`).
- Ключевые пункты: shortcut-логи временно заглушены для снижения шума в консоли (`src-tauri/src/shortcuts.rs:15`).
- Риски/вопросы: mute shortcut-логов ухудшает разбор регрессий hotkeys в runtime.
- Риски/вопросы: требуется подтверждение эффекта warm-up/pooling по p50/p95 `first_chunk_ms`.
- Следующие шаги: замерить p50/p95 `first_chunk_ms` на холодном/тёплом старте и зафиксировать baseline.
- Следующие шаги: прогнать multi-turn smoke-test на релевантность для связанных вопросов без повторов.
- Следующие шаги: вернуть shortcut-логи под управляемый debug-флаг после стабилизации.

## 2026-04-02
- Ключевые пункты: screenshot-логика вынесена в общий модуль `src/lib/screenshot/`, hooks `useCompletion` и `useChatCompletion` оставлены раздельными.
- Ключевые пункты: capture pipeline теперь учитывает visibility окна до старта, восстанавливает позицию и не показывает окно повторно, если оно уже было скрыто.
- Ключевые пункты: screenshot debug-логи спрятаны за `VITE_SCREENSHOT_DEBUG`, шум по умолчанию убран.
- Ключевые пункты: для OpenAI short-mode добавлен отдельный screenshot token budget (`800`) и screenshot-aware prompt override для coding-задач.
- Риски/вопросы: нужен ручной QA на Linux WM/композиторах, особенно для fallback-capture и restore position.
- Риски/вопросы: требуется проверить, что screenshot coding-ответы стали полнее без деградации обычного short-mode.
- Следующие шаги: прогнать manual/auto screenshot smoke-test на Linux.
- Следующие шаги: проверить сценарий «окно скрыто до capture» и «окно видно до capture».
- Следующие шаги: протестировать 3–5 screenshot coding-задач в `short` и зафиксировать результат.

## 2026-04-03
- Ключевые пункты: для main overlay добавлен runtime passive mode state в backend Tauri с командами чтения/записи (`src-tauri/src/window.rs:13`).
- Ключевые пункты: passive mode интегрирован в существующую shortcut-систему как `toggle_passive_mode` с дефолтом `Cmd/Ctrl+Shift+P` (`src/config/shortcuts.ts:85`).
- Ключевые пункты: frontend синхронизирует состояние режима через `get_main_window_passive` и событие `passive-mode-changed`, без `localStorage`.
- Ключевые пункты: в overlay добавлен компактный индикатор `Passive / Interactive` (`src/pages/app/index.tsx:78`).
- Риски/вопросы: по ручному тестированию на macOS переключение passive/interactive через shortcut приводит к падению приложения.
- Риски/вопросы: пока не решено, нужно ли сохранять passive mode между запусками.
- Следующие шаги: воспроизвести и локализовать crash на macOS при toggle passive mode через hotkey.
- Следующие шаги: проверить toggle passive mode в сценариях click-through, screenshot capture и show/hide окна.
- Следующие шаги: убедиться, что shortcut действительно конфигурируется через существующий dashboard UI без регрессий.

## 2026-04-06
- Ключевые пункты: принято решение временно заморозить runtime toggle passive mode на macOS ради стабильности window lifecycle.
- Ключевые пункты: backend passive-mode path на macOS переведён в безопасный degraded state без runtime-переключения.
- Ключевые пункты: passive mode control убран из overlay UI на macOS, чтобы не показывать нерабочий runtime-state.
- Ключевые пункты: shortcut `Toggle passive mode` сохранён в settings, но явно отключён/недоступен на macOS.
- Ключевые пункты: frontend shortcut guard больше не вызывает misleading passive-mode toggle на macOS.
- Риски/вопросы: точная panel-safe стратегия для возврата passive mode пока не выбрана.
- Риски/вопросы: отдельной проверки требует `opacity` и общее поведение окна на macOS в hotkey flow.
- Следующие шаги: стабилизировать `opacity` и сценарии show/hide/screenshot на macOS.
- Следующие шаги: подготовить критерии приёмки для возвращения passive mode без регрессий.
- Следующие шаги: вернуть диагностируемость hotkeys/window behavior под управляемый debug-путь.

## 2026-04-06
- Ключевые пункты: найдена причина краша opacity на macOS — повторный runtime `to_panel()` для уже panelized main window.
- Ключевые пункты: реализован безопасный доступ к существующему panel через `get_webview_panel(...)` вместо повторной panel re-creation.
- Ключевые пункты: runtime re-panelization убран из backend opacity path на macOS.
- Ключевые пункты: для transparency slider на macOS добавлен commit-only update вместо live native update во время drag.
- Риски/вопросы: поведение panel при нестандартных lifecycle состояниях всё ещё требует отдельной проверки.
- Риски/вопросы: нужна дальнейшая стабилизация общего window lifecycle на macOS.
- Следующие шаги: проверить stability в сценариях hide/show и screenshot flow.
- Следующие шаги: продолжить работу над macOS window layer и panel state.

## 2026-04-13
- Ключевые пункты: keyboard-first UX overlay улучшен без расширения архитектуры shortcut-системы.
- Ключевые пункты: `Esc` больше не прячет response panel и теперь работает как soft-cancel/blur для completion UI.
- Ключевые пункты: response panel теперь можно надёжно вернуть shortcut’ом `toggle_response_panel`.
- Ключевые пункты: для длинных ответов добавлены shortcuts на scroll response viewport вверх/вниз.
- Ключевые пункты: `refocus input box` снова работает, включая сценарий после переключения в другое приложение.
- Риски/вопросы: на macOS refocus ощущается более жёстким app-switch, чем обычный reopen overlay; это backlog на polish.
- Риски/вопросы: keyboard-only overlay UX ещё нужно прогнать в нескольких реальных сценариях hide/show и длинных ответов.
- Следующие шаги: отдельно отполировать более мягкий macOS refocus без лишнего ощущения stealing focus.
- Следующие шаги: прогнать keyboard-only smoke-test для `Esc`, `toggle_response_panel`, scroll shortcuts и `refocus input`.
- Следующие шаги: продолжить стабилизацию общего window lifecycle на macOS без возврата к unsafe panel/window path.

## 2026-04-30
- Ключевые пункты: добавлен override стартового passive mode через переменную окружения `PLUELY_PASSIVE_DEFAULT=false`.
- Ключевые пункты: `PassiveModeState::default()` и `setup_main_window` используют общее стартовое значение, чтобы state не перезаписывался старым macOS-дефолтом.
- Ключевые пункты: инициализация macOS panel теперь берёт `set_ignore_mouse_events(...)` из backend state, поэтому dev-запуск может быть интерактивным.
- Риски/вопросы: override не возвращает переключение passive mode во время работы; `Cmd+Shift+P` на macOS остаётся замороженным до panel-safe реализации.
- Следующие шаги: проверить вручную `PLUELY_PASSIVE_DEFAULT=false npm run tauri dev` на macOS и обычный запуск без env.
