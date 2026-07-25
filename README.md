# SpeedReader for Zed

RSVP-оверлей для быстрого чтения в Zed editor. На Space — пауза и прыжок на позицию в редакторе.

## Быстрый старт

### 1. Собери и установи GUI-бинарник

```sh
cargo build --release -p speed-reader-gui
cp target/release/speed-reader-gui /usr/local/bin/speed-reader
# или: cargo install --path gui
```

### 2. Установи расширение Zed

```sh
Cmd+Shift+P → "zed: install dev extension" → выбери папку zed-ext/
```

### 3. Используй

  {
    "context": "Editor",
    "bindings": {
      "cmd-shift-r": ["task::Spawn", { "task_name": "SpeedReader: Read current file" }]
    }
  }
[
  {
    "context": "Editor",
    "bindings": {
      "cmd-shift-r": "task::Spawn",
      "cmd-shift-r": ["task::Spawn", { "task_name": "SpeedReader: Read current file" }]
    }
  }
]
```

Теперь по `Cmd+Shift+R` сразу открывается SpeedReader с текущим файлом.

Или по старинке: выдели текст → `Cmd+C` → в терминале `speed-reader`.

## Управление

| Клавиша | Действие |
|---------|----------|
| `Space` | Пауза + прыжок на позицию в Zed |
| `Space` (ещё) | Продолжить чтение |
| `Esc` | Закрыть оверлей |
| `←` `→` | Перемотка |
| `↑` `↓` | Скорость ±10 WPM |
| `S` | Настройки |
| `ЛКМ drag` | Перетащить окно |

## Структура

```
core/   RSVP-движок (токены, ORP, тайминг, позиция, конфиг)
gui/    Оверлейное окно (winit + ab_glyph)
zed-ext/  Расширение Zed (WASM)
```
