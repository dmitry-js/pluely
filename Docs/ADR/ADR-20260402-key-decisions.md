# ADR: Централизация screenshot-pipeline и screenshot-aware политика ответа

## Контекст
После доработок screenshot-mode возникли две связанные проблемы:
1. логика захвата экрана и восстановления окна дублировалась в `useCompletion` и `useChatCompletion`;
2. screenshot coding-задачи в OpenAI `/v1/responses` обрывались из-за short-mode policy и ultra-short prompt.

## Решения
1. Вынести общую screenshot-логику в `src/lib/screenshot/`: константы, builder запроса и shared helper `captureWithoutOverlay`.
2. Сделать screenshot capture visibility-aware: окно скрывается перед захватом, позиция восстанавливается только если окно было видно до старта, а hidden-state пользователя сохраняется.
3. Для screenshot-mode использовать stateless request-path: предыдущая история не передаётся, изображения становятся главным контекстом, а при нескольких screenshots автоматически добавляется internal hint, что это одна задача.
4. Для OpenAI short-mode ввести screenshot-aware policy: отдельный token budget для `screenshot_mode=true` и дополнительный prompt override для coding-задач по скриншоту.
5. Зафиксировать screenshot-aware prompt v2 как отдельный слой поведения для coding screenshots: цель слоя состоит в снижении hallucination, более аккуратной работе с кодом на screenshot и переходе к более полному ответу вместо ultra-short output. Interview-style fallback для неявных задач остаётся направлением для отдельной валидации, а не полностью гарантированным поведением.

## Альтернативы (кратко)
- Оставить дублирование в двух hooks: отклонено из-за роста риска расхождения поведения.
- Всегда показывать окно после screenshot capture: отклонено, т.к. ломает ожидание пользователя при заранее скрытом overlay.
- Продолжать использовать общий chat history для screenshot-запросов: отклонено, т.к. это вело к повторению старых screenshot-ответов и ухудшало релевантность.
- Решать обрывы только увеличением `max_output_tokens`: отклонено, т.к. ultra-short prompt продолжал давить на стиль ответа.

## Последствия
- Screenshot-flow стал централизованным и проще для дальнейшей отладки.
- Поведение окна после capture ближе к ожидаемому UX.
- Screenshot-mode перестал зависеть от предыдущих screenshot-turns и стал предсказуемее для повторных анализов.
- Screenshot coding-задачи в short-mode должны отвечать полнее без изменения глобального short-поведения для обычного чата.
- Prompt-layer для screenshot-mode стал явнее, но richer interview-style правила ещё требуют QA на фиксированном наборе задач.

## Ссылки/файлы
- `src/lib/screenshot/captureWithoutOverlay.ts:11`
- `src/lib/screenshot/buildScreenshotRequest.ts:20`
- `src/lib/screenshot/constants.ts:1`
- `src/lib/utils.ts:8`
- `src/lib/functions/ai-response.function.ts:19`
- `src/lib/functions/ai-response.function.ts:29`
- `src-tauri/src/api.rs:19`
- `src-tauri/src/api.rs:905`
- `src-tauri/src/api.rs:1103`
