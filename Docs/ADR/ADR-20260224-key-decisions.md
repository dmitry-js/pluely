# ADR: Backend-first OpenAI и speed-first управление размером ответа

## Контекст
После перевода OpenAI на backend возникли требования по скорости и стабильности:
1) исключить browser-запросы к OpenAI и не светить ключ в DevTools;
2) убрать зависимость OpenAI-пути от лицензии при пользовательском ключе;
3) снизить latency и ошибки формата за счёт сокращения контекста и явного контроля длины ответа.

## Решения
1. Для OpenAI использовать только backend-путь `invoke("chat_stream_response")`; фронтенд не ходит в `api.openai.com` напрямую.
2. Хранить OpenAI API key в backend secure storage проекта через команды `set_openai_api_key`, `get_openai_api_key_status`, `clear_openai_api_key`; не хранить ключ в `localStorage`.
3. Применить speed-first политику для OpenAI `/v1/responses`:
   - история: только `role == "user"`, лимит последних 3 сообщений;
   - `Response Length` маппится в реальные параметры API:
     `short=150/minimal`, `medium=600/medium`, `auto=800/medium`.

## Альтернативы (кратко)
- Оставить OpenAI в frontend curl-runner: отклонено из-за безопасности ключа и разрозненной оптимизации.
- Передавать полную историю (включая assistant): отклонено из-за latency и ошибок формата для `/v1/responses`.
- Управлять длиной только prompt-инструкциями: отклонено, т.к. нет жёсткого контроля выходных токенов.

## Последствия
- OpenAI-ключ не участвует в browser network и не сохраняется в frontend-хранилище.
- Ответы стали более предсказуемыми по длине и быстрее при `short`.
- Возможен компромисс качества на сложных задачах из-за жёсткого лимита истории и токенов (требуется мониторинг).

## Ссылки/файлы
- `src/lib/functions/ai-response.function.ts:122`
- `src/lib/functions/ai-response.function.ts:211`
- `src/pages/dev/components/ai-configs/Providers.tsx:43`
- `src/pages/dev/components/ai-configs/Providers.tsx:172`
- `src/contexts/app.context.tsx:245`
- `src/contexts/app.context.tsx:510`
- `src-tauri/src/api.rs:127`
- `src-tauri/src/api.rs:146`
- `src-tauri/src/api.rs:693`
- `src-tauri/src/api.rs:770`
- `src-tauri/src/api.rs:875`
- `src-tauri/src/api.rs:965`
- `src-tauri/src/lib.rs:106`
