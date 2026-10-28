# Horizon для Android / Horizon for Android (unofficial)

> 🇷🇺 Русский ниже · 🇬🇧 English further down

---

## 🇷🇺 Русский

### Что это

Это **неофициальный порт [Horizon](https://github.com/horizon-rust/horizon) на Android**. Проект — форк
репозитория https://github.com/horizon-rust/horizon, и вся основная работа — заслуга оригинальных авторов Horizon.
Огромное им спасибо! Я только добавил то, что нужно для запуска на телефоне: сборку APK, сенсорное управление,
экранную клавиатуру и пару мелочей.

Оригинальный README лежит в [README.upstream.md](README.upstream.md).

### ⚠️ Важно

- Я **никак не связан** с разработчиками Horizon, с Mojang или Microsoft. Это любительский проект.
- В репозитории и в APK **нет никаких файлов Minecraft**. Нужна своя **легальная копия Minecraft Java Edition**,
  а игровые файлы ты готовишь сам.

### Установка

1. Скачай `horizon.apk` со страницы [Releases](https://github.com/Rayan-XOO/horizon/releases).
2. Открой его на телефоне (разреши «установку из неизвестных источников») или `adb install -r horizon.apk`.
3. Нужен Android 8.0+ и arm64.

### Подготовка игровых файлов (на ПК)

Нужны Java и Rust. В папке этого репозитория:

```sh
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
cargo run --release -p horizon-prepare -- sounds     # по желанию, для звука
```

Запусти приложение на телефоне один раз (чтобы Android создал папку), потом скопируй *содержимое*
`.local/horizon-client/26.3/` (там где `horizon-resources.json`) в:

```
/sdcard/Android/data/dev.horizonlab.horizon/files/resources/
```

Например:

```sh
adb shell mkdir -p /sdcard/Android/data/dev.horizonlab.horizon/files/resources
adb push .local/horizon-client/26.3/. /sdcard/Android/data/dev.horizonlab.horizon/files/resources/
```

Подробности — в [ANDROID.md](ANDROID.md).

### Управление

- Джойстик слева для движения, свайп справа — осмотреться.
- Кнопки: прыжок, присед, атака, использование, инвентарь, смена слота.
- Пауза в правом верхнем углу, кнопка **X** закрывает открытый экран.
- Кнопки **F1 / F2 / F3 / F5** (скрыть HUD, скриншот, отладка, камера).
- Экранная клавиатура сама открывается в чате, табличках, поиске и т.д.
- Редактор раскладки (шестерёнка): двигай кнопки, меняй размер и прозрачность, сохраняй.
- Быстрый тап по мобу — атака.

### На чём проверено

Samsung Galaxy S25 (Adreno 830, Vulkan). Доходит до меню и одиночного мира, всё играбельно.

### Известные проблемы

- Ест около **2,2 ГБ ОЗУ** — на слабых телефонах может вылетать. Попробуй дальность прорисовки 4–6.
- Это **ранняя, экспериментальная** версия: баги будут. Логи — в `files/logs/latest.log`.

### Сборка

Инструкции по сборке APK — в [ANDROID.md](ANDROID.md).

### Лицензия

MIT, как и оригинал. Файл [LICENSE](LICENSE) и копирайт оригинальных авторов не тронуты.

---

## 🇬🇧 English

### What is this

This is an **unofficial Android port of [Horizon](https://github.com/horizon-rust/horizon)**. It's a fork of
https://github.com/horizon-rust/horizon, and almost all the real work belongs to the original Horizon authors —
huge thanks to them! I just added what's needed to run it on a phone: APK packaging, touch controls, an on-screen
keyboard and a few small things.

The original README lives in [README.upstream.md](README.upstream.md).

### ⚠️ Heads up

- I'm **not affiliated** with the Horizon developers, Mojang or Microsoft. This is a hobby project.
- **No Minecraft files are included** in this repo or the APK. You need your own **legit copy of Minecraft Java
  Edition** and you prepare the game files yourself.

### Install

1. Grab `horizon.apk` from [Releases](https://github.com/Rayan-XOO/horizon/releases).
2. Open it on your phone (allow "install unknown apps") or run `adb install -r horizon.apk`.
3. Needs Android 8.0+ and an arm64 device.

### Preparing game files (on a PC)

You'll need Java and Rust. From this repo:

```sh
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
cargo run --release -p horizon-prepare -- sounds     # optional, for sound
```

Launch the app once (so Android creates its folder), then copy the *contents* of `.local/horizon-client/26.3/`
(the folder with `horizon-resources.json`) to:

```
/sdcard/Android/data/dev.horizonlab.horizon/files/resources/
```

For example:

```sh
adb shell mkdir -p /sdcard/Android/data/dev.horizonlab.horizon/files/resources
adb push .local/horizon-client/26.3/. /sdcard/Android/data/dev.horizonlab.horizon/files/resources/
```

More details in [ANDROID.md](ANDROID.md).

### Controls

- Joystick on the left to move, swipe on the right to look around.
- Buttons: jump, sneak, attack, use, inventory, next slot.
- Pause in the top-right corner, **X** closes the open screen.
- **F1 / F2 / F3 / F5** buttons (hide HUD, screenshot, debug, camera).
- The on-screen keyboard pops up by itself in chat, signs, search fields, etc.
- Layout editor (gear icon): move buttons, resize them, change opacity, save.
- Quick tap on a mob attacks it.

### Tested on

Samsung Galaxy S25 (Adreno 830, Vulkan). Gets to the menus and a single-player world, and it's playable.

### Known issues

- Uses around **2.2 GB of RAM** — weaker phones may crash. Try render distance 4–6.
- This is an **early, experimental** build, so expect bugs. Logs are in `files/logs/latest.log`.

### Building

APK build instructions are in [ANDROID.md](ANDROID.md).

### License

MIT, same as the original. The [LICENSE](LICENSE) file and the original authors' copyright are left untouched.

---

— Порт собран с помощью Grok Bot 🤖 / Port built with the help of Grok Bot

Автор порта / Port author: [Rayan-XOO](https://github.com/Rayan-XOO)
