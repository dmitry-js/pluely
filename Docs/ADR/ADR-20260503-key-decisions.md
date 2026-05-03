# ADR: System audio Pause и направление STT proxy

## Контекст
- System audio capture использует backend Rust/VAD и эмитит `speech-detected` сегменты во frontend.
- Pause/Resume нужен для interview flow, но остановка capture session должна оставаться отдельным явным действием.
- Прототип backend flush на Pause не дал очевидного улучшения tail capture и усложнял VAD state machine.
- Текущий direct desktop client -> OpenAI STT path нестабилен для пользователей за VPN/restricted network: возможны network timeout/connect/send ошибки.

## Решения
1. Не сохранять `flush_system_audio_segment` и связанный backend flush-state.
2. Оставить Pause простым frontend gate: `PAUSE_GRACE_MS = 2500`, STT queue, pending transcript buffer и ручной `Cmd+Enter` flow.
3. Считать минимальный STT proxy backend приоритетным направлением: клиент отправляет audio в backend Pluely, backend проксирует STT provider, обрабатывает timeout/retry/fallback и возвращает transcript.

## Альтернативы
- Продолжать backend-level VAD flush сейчас: отложено до измеримого подтверждения пользы на сохранённых WAV-сегментах.
- Добавлять новые frontend timing heuristics для Pause: отклонено, так как поведение становится слишком «магическим».
- Оставить direct desktop client -> OpenAI STT как основной путь: недостаточно надёжно для пользователей с restricted network/VPN.

## Последствия
- Pause не останавливает audio capture на backend; он продолжает захват и сегментацию, но frontend решает, принимать ли сегменты в STT pipeline.
- `PLUELY_SAVE_STT_SEGMENTS` остаётся dev-only диагностикой для прослушивания emitted VAD segments.
- Надёжность STT нужно улучшать через backend proxy, а не через дополнительные pause heuristics.

## Ссылки/файлы
- `src/hooks/useSystemAudio.ts`
- `src-tauri/src/speaker/commands.rs`
- `src/lib/functions/stt.function.ts`
- `src-tauri/src/api.rs`
- `Docs/NOTES/todo_ru.md`
