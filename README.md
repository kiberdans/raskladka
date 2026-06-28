# raskladka

Переключение раскладки клавиатуры (QWERTY ↔ ЙЦУКЕН) по двойному нажатию
настраиваемой клавиши (по умолчанию Shift).

## Установка

### Debian / Ubuntu

```bash
sudo apt install xdotool xclip curl
sudo dpkg -i raskladka_0.2.0_amd64.deb
systemctl --user enable --now raskladka
```

### Fedora / RHEL

```bash
sudo dnf install xdotool xclip curl
sudo rpm -i raskladka-0.2.0-1.x86_64.rpm
systemctl --user enable --now raskladka
```

### Arch Linux

```bash
sudo pacman -S xdotool xclip wl-clipboard wtype curl
git clone https://github.com/kiberdans/raskladka
cd raskladka
makepkg -si
systemctl --user enable --now raskladka
```



### AppImage (универсальный)

```bash
curl -L -o raskladka.AppImage https://github.com/kiberdans/raskladka/releases/download/v0.2.0/raskladka-x86_64.AppImage
chmod +x raskladka.AppImage
./raskladka.AppImage
```

### Сборка из исходников

```bash
git clone https://github.com/kiberdans/raskladka
cd raskladka
cargo build --release
sudo cp target/release/raskladka /usr/local/bin/
```

## Права доступа

Для перехвата нажатий клавиш нужен доступ к `/dev/input/*`.
Добавьте себя в группу `input` и перелогиньтесь:

```bash
sudo usermod -aG input $USER
```

Без этого можно запускать с `sudo`.

## Использование

1. Выделите текст
2. Дважды быстро нажмите Shift
3. Раскладка переключится

**Трей:** ЛКМ — вкл/выкл, ПКМ — меню, наведение — тултип.
**Переназначение клавиши:** ПКМ → «rebind key» → нажмите любую клавишу
или кнопку мыши (кроме ЛКМ/ПКМ). Сохраняется в конфиг.
**Язык интерфейса:** ПКМ → «language» — переключает En/Ru (с рестартом).
**Автозапуск:** ПКМ → «start on login» — создаёт/удаляет
`~/.config/autostart/raskladka.desktop`.
**Обновления:** ПКМ → «check updates» — автоматическая проверка
каждый час (через GitHub API). При наличии новой версии появляется
пункт «update vX.Y.Z !» — открывает страницу релиза.

## CLI

```bash
raskladka        # запуск демона с треем
raskladka status # печатает "on" или "off"
raskladka toggle # переключает вкл/выкл (полезно для хоткеев DE)
```

## Polybar

Добавьте в `~/.config/polybar/config`:

```ini
[module/raskladka]
type = custom/script
exec = raskladka status
interval = 2
click-left = raskladka toggle
format-prefix = "⌨ "
format-prefix-foreground = ${colors.primary}
```

## Конфигурация

Файл `~/.config/raskladka/config` создаётся автоматически при первом
переназначении клавиши или смене языка через меню трея. Пример:

```
trigger=ShiftLeft
timeout_ms=400
lang=en
check_updates=true
autostart=true
```

- `trigger` — имя варианта из
  [Key](https://docs.rs/rdev/0.5.3/rdev/enum.Key.html) (`ControlLeft`,
  `Alt`, `F1`...) или `m:Middle` для средней кнопки мыши
- `timeout_ms` — таймаут двойного нажатия (по умолчанию 400)
- `lang` — язык интерфейса: `en` или `ru`
- `check_updates` — проверять обновления каждый час (`true`/`false`)
- `autostart` — создавать `~/.config/autostart/raskladka.desktop` (`true`/`false`)

## Ссылки

- [Releases](https://github.com/kiberdans/raskladka/releases)

## Лицензия

MIT
