- [x] Стабилизировать native `opacity` на macOS.
  - Приёмка: изменение opacity больше не приводит к crash в settings slider.
  - Приёмка: runtime update использует существующий panel без повторного `to_panel()`.

- [ ] Harden macOS window lifecycle (`opacity` + panel state).
  - Приёмка: проверки проходят для hide/show и быстрых изменений состояния окна.
  - Приёмка: нет регрессий при повторных runtime update и в нестандартных panel state.

- [ ] Отполировать keyboard-first window behavior на macOS.
  - Приёмка: `refocus input box` возвращает ввод без ощущения жёсткого app switch / лишнего stealing focus.
  - Приёмка: keyboard-only сценарии overlay (`Esc`, `toggle_response_panel`, scroll response, refocus input) проходят без заметных UX-регрессий.

- [ ] Подготовить panel-safe стратегию возврата passive mode на macOS.
  - Приёмка: есть краткая заметка с выбранным подходом и списком API/ограничений.
  - Приёмка: решение не опирается на небезопасный runtime toggle через текущий lifecycle окна.

- [ ] Восстановить passive mode на macOS после стабилизации окна.
  - Приёмка: runtime toggle работает без crash в `Cmd+Shift+P`.
  - Приёмка: UI overlay и settings снова честно отражают доступность функции.

- [ ] Вернуть управляемую диагностику hotkeys/window behavior.
  - Приёмка: shortcut/window debug можно включать без лишнего шума по умолчанию.
  - Приёмка: по логам можно разобрать passive mode, opacity и screenshot hotkey flow.

- [ ] Прогнать screenshot-flow smoke-test после window stabilization.
  - Приёмка: `visible -> capture -> restore` и `hidden -> capture -> stay hidden` проходят без регрессий.
  - Приёмка: overlay не попадает в screenshot и окно возвращается в ожидаемое состояние.

- [ ] Вернуться к backend-level pause/flush для system audio как отдельному исследованию.
  - Текущее поведение: Pause реализован на frontend как gate для `speech-detected` с grace window `PAUSE_GRACE_MS = 2500`; STT queue и ручной `Cmd+Enter` flow остаются без изменений.
  - Важно: backend VAD во время Pause продолжает захватывать аудио и может эмитить сегменты; frontend решает, принять их через grace window или пропустить.
  - Контекст: прототип `flush_system_audio_segment` не оставлен, потому что практическая польза была неочевидна, а сложность VAD state machine росла.
  - Приёмка будущего решения: понятные метрики улучшения tail capture, отсутствие лишних коротких/hallucination-prone сегментов и минимальное усложнение VAD.
