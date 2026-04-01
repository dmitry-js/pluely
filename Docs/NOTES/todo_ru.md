- [ ] Прогнать QA screenshot-flow на Linux.
  - Приёмка: сценарии `visible -> capture -> restore` и `hidden -> capture -> stay hidden` проходят без регрессий.
  - Приёмка: overlay не попадает в итоговый screenshot.

- [ ] Проверить manual/auto screenshot mode после рефакторинга.
  - Приёмка: в manual mode screenshot добавляется в attachments и отправляется кнопкой/shortcut.
  - Приёмка: в auto mode screenshot сразу уходит в stateless AI-request.

- [ ] Валидировать screenshot coding tasks в `short`.
  - Приёмка: минимум 3 сценария (например LeetCode / TS utility types / bugfix по screenshot) возвращают полный код без обрыва.
  - Приёмка: обычные short-text запросы остаются краткими.

- [ ] Проверить fallback-capture на Linux compositor.
  - Приёмка: при пустом hidden-capture второй capture с visible window даёт валидный кадр.
  - Приёмка: нет крашей/`undefined` по `base64`.

- [ ] Протестировать screenshot prompt v2 на interview-style задачах.
  - Приёмка: screenshots с `async` / recursion / event loop анализируются без грубых фактических ошибок.
  - Приёмка: отдельно проверены сценарии с microtasks/macrotasks и выводом о блокировке UI.

- [ ] Провести baseline testing screenshot-mode на фиксированном наборе кейсов.
  - Приёмка: собран набор минимум из 10 screenshot-кейсов и для каждого зафиксирована оценка качества ответа.
  - Приёмка: результаты разделены минимум на coding / UI / mixed-context сценарии.

- [ ] Оценить 2-stage pipeline `extract -> answer` как будущую опцию.
  - Приёмка: есть короткая заметка с плюсами/рисками и явной пометкой, что это не текущий приоритет.
  - Приёмка: решение не влияет на текущий stateless screenshot-flow.
