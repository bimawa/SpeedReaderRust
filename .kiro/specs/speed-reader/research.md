# Research Log: SpeedReader для Zed

## Summary

Исследование показало, что текущее Zed Extension API **не поддерживает** создание кастомных UI-оверлеев, манипуляцию позицией курсора, чтение активного буфера (только чтение файлов из worktree) или регистрацию кастомных команд редактора. Это фундаментальное ограничение, которое меняет архитектуру решения.

Основные выводы:
1. **Zed Extension API** предназначен для языковых серверов, тем, сниппетов, отладчиков и MCP-серверов — НЕ для кастомных UI-панелей
2. **RSVP-движок** может быть чистым Rust/WASM, но отображение требует внешнего GUI
3. **Оптимальная архитектура**: ядро на Rust (компилируется и в WASM, и нативно) + GUI через отдельное приложение

---

## Research Log

### Topic 1: Обзор Zed Extension API
- **Source**: https://zed.dev/docs/extensions/developing-extensions.md, WIT-файлы v0.8.0
- **Finding**: API предоставляет: LSP, DAP, slash-команды, контекст-серверы, HTTP-клиент, работу с процессами, GitHub API, Node.js, docs-индексацию, key-value store, чтение файлов из worktree.
- **Impact**: Критическое ограничение — нет API для UI, буфера, курсора, команд.

### Topic 2: RSVP и SpeedReader
- **Source**: https://speed-reader.pro/, https://github.com/one-more-refactor/flick, https://github.com/matheusmendes720/rsvp-tui
- **Finding**: SpeedReader.pro — нативное macOS приложение (Swift). flick-core и rsvp-core — существующие Rust RSVP-движки (но с GPL-лицензиями). Формула ORP и WPM-тайминга хорошо документирована.
- **Impact**: ORP-алгоритм и WPM-тайминг можно реализовать на Rust с MIT-лицензией.

### Topic 3: WIT-спецификация Extension API
- **Source**: https://github.com/zed-industries/zed/blob/main/crates/extension_api/wit/
- **Finding**: Вся спецификация API в WIT. Нет методов для UI, буфера, курсора, команд.
  - `worktree.read_text_file(path)` — только чтение с диска
  - `project.worktree_ids()` — только ID рабочих деревьев
  - `run-slash-command` — только статический текст в Assistant
- **Impact**: Расширение не может получить доступ к открытому буферу или создать оверлей.

### Topic 4: Архитектурные опции для GUI
- **Source**: Анализ gpui, winit, skia-safe, egui
- **Finding**: gpui — внутренний фреймворк Zed, не экспортируется. Для оверлея: `winit` + `skia-safe` (лёгкий, GPU-текст) или `egui` (немедленный рендеринг).
- **Impact**: Выбор — `winit` + `skia-safe` для минимального оверлейного окна.

### Topic 5: Существующие Rust RSVP-реализации
- **Source**: flick-core (AGPL-3.0), rsvp-core (GPL-3.0), readfastrs (Leptos/WASM), nabu (Leptos/WASM)
- **Finding**: Два основательных RSVP-движка на Rust, но с проблемными лицензиями. Алгоритмы ORP, WPM-scaling, chunking доступны для изучения.
- **Impact**: Использовать паттерны (не код) для чистой реализации.

---

## Architecture Pattern Evaluation

| Pattern | Pros | Cons | Verdict |
|---------|------|------|---------|
| Pure Rust/WASM Extension | Простая установка, родная интеграция | Нет UI, нет курсора, нет буфера | ❌ Недостаточно |
| Slash Command в Assistant | Встроено в API | Статичный текст, нет анимации, только Assistant | ❌ Не подходит |
| Внешнее GUI приложение + Zed Extension | Полный контроль UI, Rust-native | Требует установки двух компонентов, IPC | ✅ Рабочий вариант |
| Fork Zed + кастомное API | Полный контроль | Сложность поддержки, непубликуемо | ❌ Слишком сложно |
| Feature Request в Zed API | Долгосрочное решение | Неопределённые сроки | ⚠️ Параллельно |
| **Hybrid: Core lib + GUI overlay + thin Zed ext** | **Модульность, поэтапная реализация** | **Два компонента для установки** | **✅ Рекомендуется** |

## Design Decisions

| Decision | Options | Selected | Rationale |
|----------|---------|----------|-----------|
| Язык ядра | Rust / TypeScript / C | Rust | WASM-компиляция, приоритет пользователя |
| GUI фреймворк | winit+skia / egui / webview | winit + skia-safe | Минимальный оверхей, GPU-текст |
| RSVP токенизация | unicode-segmentation / regex / custom | unicode-segmentation + custom | Стандартная графемная сегментация |
| ORP алгоритм | flick-style / spritz-style | flick-style | Проверенная формула с word-length scaling |
| IPC | stdin/stdout / unix socket / temp file | stdin/stdout | Простейший, кроссплатформенный |
| Конфигурация | JSON / TOML / hardcoded | JSON | Человекочитаемый, редактируемый |
| Zed интеграция | Slash command / Context server / External | External launch + keyboard shortcut | Slash command ограничен Assistant'ом |

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Zed API не имеет UI/оверлей | Critical | Confirmed | Архитектура Core + GUI overlay |
| Нет доступа к активному буферу | High | Confirmed | Чтение через clipboard + file path |
| Нет доступа к курсору | High | Confirmed | Навигация по отчёту позиции |
| RSVP снижает понимание | Medium | Confirmed (study) | Pause/rewind controls, user education |
| Глазная усталость от RSVP | Medium | Confirmed (study) | Ограничение сессии, частые паузы |

## Open Questions
1. Есть ли планы у Zed team добавить UI Panel API в extensions? — Нужно мониторить `zed-industries/zed` репозиторий и issues
2. Какой IPC-механизм выбрать для стабильной работы? — stdin/stdout как стартовый вариант
3. Можно ли использовать `editor::MoveTo` через keybinding proxy? — Нет прямого API, но можно эмулировать через clipboard

## Synthesis Outcomes

### 1. Generalization
RSVP-движок — чистый data-трансформ: `text -> token stream -> timed display events`. Pipeline переиспользуется:
- В GUI-приложении (нативная компиляция)
- В WASM (если Zed API расширится)
- В CLI для pipe-обработки
- В браузерном WASM

Позиция чтения — общая абстракция: `token_index -> source_byte_offset`. Единая для всех платформ.

### 2. Build vs Adopt
- RSVP engine: **Build** (MIT) — flick-core AGPL-3.0, rsvp-core GPL-3.0. Использовать паттерны из них.
- ORP algorithm: **Build** — формула известна: `index = min((len+1)/2 - 1, 3)` для слов > 2 букв.
- Window overlay: **Build** (winit + skia-safe) — одно окно с текстом, не нужен полный GUI.
- Zed extension: **Build** (WASM) — уникальная интеграция.
- IPC: **Adopt** (stdin/stdout) — тривиально, кроссплатформенно.

### 3. Simplification
- GUI не требует фулстек-фреймворка. Оверлей — одно прозрачное окно.
- Конфигурация — простой JSON-файл.
- Расширение Zed — минимальная прослойка (только коммуникация).
- Состояние: конечный автомат `Stopped | Playing | Paused | PositionSynced`.
- Не нужно MCP/context server — только slash-команда + внешний launch.
