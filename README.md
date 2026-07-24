# SpeedReader for Zed

RSVP-оверлей для быстрого чтения в Zed editor. На Space — пауза и прыжок на позицию в редакторе.

## Быстрый старт

### 1. Собери и установи GUI-бинарник

```sh
# Сборка
cargo build --release -p speed-reader-gui

# Установка (бинарник в ~/.cargo/bin/)
cp target/release/speed-reader-gui /usr/local/bin/speed-reader
# или:
cargo install --path gui
```

### 2. Установи расширение Zed

```sh
# 1. Открой Zed
# 2. Cmd+Shift+P → "zed: install dev extension"
# 3. Выбери папку `zezed-ext/` в этом проекте
```

Или через `extension.toml` вручную, если не работает:

```sh
ln -sf "$(pwd)/zed-ext" ~/.config/zed/extensions/dev/speed-reader
```

### 3. Проверь

```sh
# Убедись что бинарник работает
speed-reader --help

# Запусти на тестовом файле
echo "Hello world speed reading test" > /tmp/test.txt
speed-reader /tmp/test.txt
# Должно открыться прозрачное окно 600x200
# Space — пауза + прыжок в Zed
# Esc — выход
# Стрелки — перемотка
# Up/Down — скорость
```

### 4. Используй в Zed

Открой файл → Cmd+Shift+P → `/speed-reader` (slash-команда в Assistant).

Или быстрее: выдели текст → Cmd+C → в терминале `speed-reader`.

## Управление

| Клавиша | Действие |
|---------|----------|
| `Space` | Пауза + прыжок на позицию в Zed |
| `Space` (ещё) | Продолжить чтение |
| `Esc` | Закрыть оверлей |
| `←` `→` | Перемотка на 5 слов |
| `↑` `↓` | Скорость ±10 WPM |
| `R` | Перезапуск |

## Настройки

Конфиг: `~/.config/speed-reader/config.json`

```sh
speed-reader --wpm 500 --theme light /tmp/test.txt
```

## Сборка расширения Zed (WASM)

```sh
rustup target add wasm32-wasip1
cargo build --release -p speed-reader-zed --target wasm32-wasip1
```

## Структура проекта

```
core/src/           # RSVP-движок (чистый Rust)
  tokenizer.rs      # Разбивка текста + ORP
  timing.rs         # Расчёт времени (WPM)
  state.rs          # Конечный автомат
  position.rs       # Маппинг позиции
  config.rs         # Модель конфига

gui/src/            # Оверлейное окно (winit + skia)
  main.rs           # CLI + запуск
  overlay.rs        # Winit окно + event loop
  renderer.rs       # Skia-рендеринг
  input.rs          # Клавиатура + zed CLI
  config.rs         # Загрузка/сохранение config.json

zed-ext/src/        # Расширение Zed (WASM)
  lib.rs            # Slash-команда
```
