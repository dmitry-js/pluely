- [ ] Исправить `opacity` на macOS.
  - Приёмка: изменение opacity не ломает show/hide и не даёт артефактов у overlay.
  - Приёмка: smoke-test проходит для сценариев startup, dashboard open, screenshot flow.

- [ ] Стабилизировать window behavior на macOS в hotkey flow.
  - Приёмка: сценарии hide/show, screenshot shortcut и dashboard toggle не приводят к падению приложения.
  - Приёмка: зафиксирован короткий список воспроизводимых и невоспроизводимых кейсов.

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
