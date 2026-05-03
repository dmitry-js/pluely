- [ ] Реализовать минимальный STT proxy backend (v1).
  - Приёмка: HTTP endpoint принимает audio (`multipart` или `base64`), проксирует в OpenAI Whisper или другой STT provider и возвращает transcript + нормализованные ошибки.
  - Приёмка: retry работает для network errors (`timeout/connect/send`), а overhead latency не превышает примерно +300–500ms к прямому вызову.

- [ ] Проверить STT WAV debug dump на реальных interview-сегментах.
  - Приёмка: по файлам из `PLUELY_SAVE_STT_SEGMENTS=true npm run tauri dev` понятно, где проблема: VAD segmentation, Pause gate или STT hallucination.
  - Приёмка: после длинных dev-сессий временная папка `pluely-stt-segments` удаляется вручную.

- [ ] Вернуться к backend-level pause/flush для system audio только как отдельному исследованию.
  - Приёмка: есть измеримые метрики улучшения tail capture и нет лишних коротких hallucination-prone сегментов.
  - Приёмка: решение не усложняет VAD state machine без доказанной пользы.

- [ ] Harden macOS window lifecycle (`opacity` + panel state).
  - Приёмка: проверки проходят для hide/show и быстрых изменений состояния окна.
  - Приёмка: нет регрессий при повторных runtime update и в нестандартных panel state.

- [ ] Подготовить panel-safe стратегию возврата passive mode на macOS.
  - Приёмка: есть краткая заметка с выбранным подходом и списком API/ограничений.
  - Приёмка: решение не опирается на небезопасный runtime toggle через текущий lifecycle окна.

- [ ] Вернуть управляемую диагностику hotkeys/window behavior.
  - Приёмка: shortcut/window debug можно включать без лишнего шума по умолчанию.
  - Приёмка: по логам можно разобрать passive mode, opacity и screenshot hotkey flow.

- [ ] Прогнать screenshot-flow smoke-test после window stabilization.
  - Приёмка: `visible -> capture -> restore` и `hidden -> capture -> stay hidden` проходят без регрессий.
  - Приёмка: overlay не попадает в screenshot и окно возвращается в ожидаемое состояние.
