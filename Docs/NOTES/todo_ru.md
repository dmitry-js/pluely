- [ ] Протестировать passive/interactive mode на macOS и исправить crash при toggle через shortcut.
  - Приёмка: переключение через `Cmd+Shift+P` не приводит к падению приложения.
  - Приёмка: в passive mode окно остаётся click-through, в interactive mode возвращается normal mouse interaction.

- [ ] Проверить manual/auto screenshot mode после рефакторинга.
  - Приёмка: в manual mode screenshot добавляется в attachments и отправляется кнопкой/shortcut.
  - Приёмка: в auto mode screenshot сразу уходит в stateless AI-request.

- [ ] Прогнать screenshot-flow после добавления passive mode.
  - Приёмка: сценарии `visible -> capture -> restore` и `hidden -> capture -> stay hidden` проходят без регрессий.
  - Приёмка: overlay не попадает в итоговый screenshot и passive mode не ломает capture pipeline.

- [ ] Валидировать screenshot coding tasks в `short`.
  - Приёмка: минимум 3 сценария (например LeetCode / TS utility types / bugfix по screenshot) возвращают полный код без обрыва.
  - Приёмка: обычные short-text запросы остаются краткими.

- [ ] Протестировать screenshot prompt v2 на interview-style задачах.
  - Приёмка: screenshots с `async` / recursion / event loop анализируются без грубых фактических ошибок.
  - Приёмка: отдельно проверены сценарии с microtasks/macrotasks и выводом о блокировке UI.

- [ ] Провести baseline testing screenshot-mode на фиксированном наборе кейсов.
  - Приёмка: собран набор минимум из 10 screenshot-кейсов и для каждого зафиксирована оценка качества ответа.
  - Приёмка: результаты разделены минимум на coding / UI / mixed-context сценарии.

- [ ] Проверить fallback-capture на Linux compositor.
  - Приёмка: при пустом hidden-capture второй capture с visible window даёт валидный кадр.
  - Приёмка: нет крашей/`undefined` по `base64`.

- [ ] Оценить 2-stage pipeline `extract -> answer` как будущую опцию.
  - Приёмка: есть короткая заметка с плюсами/рисками и явной пометкой, что это не текущий приоритет.
  - Приёмка: решение не влияет на текущий stateless screenshot-flow.
