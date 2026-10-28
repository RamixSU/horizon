# Horizon для Android / Horizon for Android (unofficial)

Русский ниже, English further down.

**Сырой проект, требует много допила.** Работает, но многое ещё не доделано и не отполировано.
**Raw, early project that still needs a lot of polishing.** It runs, but plenty is unfinished.

---

## Русский

### Что это

Это неофициальный порт [Horizon](https://github.com/horizon-rust/horizon) на Android. Проект — форк
репозитория https://github.com/horizon-rust/horizon, и вся основная работа - заслуга оригинальных авторов Horizon.
Спасибо им. Я только добавил то, что нужно для запуска на телефоне: сборку APK, сенсорное управление,
экранную клавиатуру и пару мелочей.

Оригинальный README лежит в [README.upstream.md](README.upstream.md).

### Важно

- Я **никак не связан** с разработчиками Horizon, с Mojang или Microsoft. Это любительский проект.
- В репозитории и в APK **нет никаких файлов Minecraft**. Нужна своя **легальная копия Minecraft Java Edition**,
  а игровые файлы ты готовишь сам.

### Установка

1. Скачай `horizon.apk` со страницы [Releases](https://github.com/Rayan-XOO/horizon/releases).
2. Открой его на телефоне (разреши «установку из неизвестных источников») или `adb install -r horizon.apk`.
3. Нужен Android 8.0+ и arm64.

### Файлы игры

При первом запуске приложение само скачает почти все файлы игры (около 525 МБ, лучше по Wi-Fi). Загрузка идёт в фоне, её можно свернуть, а если оборвётся, продолжится с того же места.

Есть одно но: небольшой архив `horizon-derived.zip` (около 4 МБ) телефон собрать сам не может, его делают инструменты Horizon на Java. Его нужно один раз сделать на ПК и закинуть на телефон. Выложить его сюда я не могу, потому что он получен из файлов игры.

**Как сделать архив (один раз, на ПК с Java 25 и Rust):**

1. Скачай этот репозиторий и открой в нём терминал.
2. Скачай `server.jar` версии 26.3 с официального сайта Minecraft и подготовь файлы игры:
   ```sh
   cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
   ```
3. Упакуй архив:
   ```sh
   bash packaging/android/make_derived_pack.sh
   ```
   Готовый файл появится в `target/android/horizon-derived.zip`.

**Как закинуть на телефон:**

1. Запусти Horizon один раз, чтобы Android создал папку игры.
2. Скопируй `horizon-derived.zip` в папку
   `/storage/emulated/0/Android/data/dev.horizonlab.horizon/files/`
   (через провод, файловый менеджер с доступом к `Android/data` или `adb push horizon-derived.zip /sdcard/Android/data/dev.horizonlab.horizon/files/`).
3. В приложении нажми «Повторить», загрузка дойдёт до конца, и игра запустится.

Если друг уже сделал архив, можно взять у него, он одинаковый для всех на версии 26.3.

Не удаляй приложение без нужды: Android при удалении стирает его папку вместе со скачанными файлами, сохранениями и архивом. Для обновления просто ставь новый APK поверх старого.

Как подготовить все файлы на ПК целиком, без загрузки на телефоне, написано в [ANDROID.md](ANDROID.md).

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

- Ест около **2,2 ГБ ОЗУ**, на слабых телефонах может вылетать. Попробуй дальность прорисовки 4-6.
- Это **ранняя, экспериментальная** версия: баги будут. Лог: `/storage/emulated/0/Android/data/dev.horizonlab.horizon/files/logs/latest.log`, отчёты о вылетах рядом в `crash-reports/`. Открывается любым файловым менеджером (на новых Android папка Android/data видна не всем менеджерам, тогда нужен менеджер с доступом к ней или `adb pull`).

### Сборка

Инструкции по сборке APK в [ANDROID.md](ANDROID.md).

### Лицензия

MIT, как и оригинал. Файл [LICENSE](LICENSE) и копирайт оригинальных авторов не тронуты.

---

## English

### What is this

This is an unofficial Android port of [Horizon](https://github.com/horizon-rust/horizon). It's a fork of
https://github.com/horizon-rust/horizon, and almost all the real work belongs to the original Horizon authors,
thanks to them. I just added what's needed to run it on a phone: APK packaging, touch controls, an on-screen
keyboard and a few small things.

The original README lives in [README.upstream.md](README.upstream.md).

### Heads up

- I'm **not affiliated** with the Horizon developers, Mojang or Microsoft. This is a hobby project.
- **No Minecraft files are included** in this repo or the APK. You need your own **legit copy of Minecraft Java
  Edition** and you prepare the game files yourself.

### Install

1. Grab `horizon.apk` from [Releases](https://github.com/Rayan-XOO/horizon/releases).
2. Open it on your phone (allow "install unknown apps") or run `adb install -r horizon.apk`.
3. Needs Android 8.0+ and an arm64 device.

### Game files

On first launch the app downloads almost all game files by itself (about 525 MB, Wi-Fi recommended). It runs in the background, and if it gets interrupted it picks up where it left off.

One catch: a small archive, `horizon-derived.zip` (about 4 MB), can't be built on the phone because Horizon makes it with Java tools. You need to build it once on a PC and copy it to the phone. I can't upload it here because it's derived from the game files.

**Building the archive (once, on a PC with Java 25 and Rust):**

1. Clone this repo and open a terminal in it.
2. Download the 26.3 `server.jar` from the official Minecraft site and prepare the game files:
   ```sh
   cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
   ```
3. Pack the archive:
   ```sh
   bash packaging/android/make_derived_pack.sh
   ```
   The file ends up in `target/android/horizon-derived.zip`.

**Copying it to the phone:**

1. Launch Horizon once so Android creates the game folder.
2. Copy `horizon-derived.zip` to
   `/storage/emulated/0/Android/data/dev.horizonlab.horizon/files/`
   (over USB, with a file manager that can open `Android/data`, or `adb push horizon-derived.zip /sdcard/Android/data/dev.horizonlab.horizon/files/`).
3. Tap "Retry" in the app. The download finishes and the game starts.

If a friend already built the archive, you can just take theirs; it's the same for everyone on 26.3.

Don't uninstall the app unless you have to: Android wipes its folder, including downloaded files, saves and the archive. To update, just install the new APK over the old one.

To prepare everything on a PC without the on-phone download, see [ANDROID.md](ANDROID.md).

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

- Uses around **2.2 GB of RAM**, weaker phones may crash. Try render distance 4-6.
- This is an **early, experimental** build, so expect bugs. Log: `/storage/emulated/0/Android/data/dev.horizonlab.horizon/files/logs/latest.log`, crash reports next to it in `crash-reports/`. Any file manager can open it (on newer Android, Android/data needs a file manager that has access to it, or use `adb pull`).

### Building

APK build instructions are in [ANDROID.md](ANDROID.md).

### License

MIT, same as the original. The [LICENSE](LICENSE) file and the original authors' copyright are left untouched.

---

Автор порта / Port author: [Rayan-XOO](https://github.com/Rayan-XOO)
