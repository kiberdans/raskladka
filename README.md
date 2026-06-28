# raskladka

Переключение раскладки клавиатуры (QWERTY ↔ ЙЦУКЕН) по двойному нажатию
настраиваемой клавиши (по умолчанию Shift).

## Установка

### Debian / Ubuntu

```bash
sudo apt install xdotool xclip
sudo dpkg -i raskladka_0.1.0_amd64.deb
sudo usermod -aG input $USER
# перелогиньтесь
systemctl --user enable --now raskladka
```

### Fedora / RHEL

```bash
sudo dnf install xdotool xclip
sudo rpm -i raskladka-0.1.0-1.x86_64.rpm
sudo usermod -aG input $USER
# перелогиньтесь
systemctl --user enable --now raskladka
```

### Arch Linux

```bash
sudo pacman -S xdotool xclip wl-clipboard wtype
git clone https://github.com/kiberdans/raskladka
cd raskladka
cargo build --release
sudo cp target/release/raskladka /usr/local/bin/
sudo usermod -aG input $USER
# перелогиньтесь
systemctl --user enable --now raskladka
```

### Универсальный установщик

```bash
curl -L -o install-raskladka.sh https://github.com/kiberdans/raskladka/releases/download/v0.1.0/install-raskladka.sh
chmod +x install-raskladka.sh
sudo ./install-raskladka.sh
```

### AppImage

```bash
curl -L -o raskladka.AppImage https://github.com/kiberdans/raskladka/releases/download/v0.1.0/raskladka-x86_64.AppImage
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

## После установки

Добавьте себя в группу `input` (иначе нужен `sudo`):

```bash
sudo usermod -aG input $USER
# перелогиньтесь
```

## Использование

1. Выделите текст
2. Дважды быстро нажмите Shift
3. Раскладка переключится

**Трей:** ЛКМ — вкл/выкл, ПКМ — меню, наведение — тултип.
**Переназначение клавиши:** ПКМ → «rebind key» → нажмите любую клавишу
или кнопку мыши (кроме ЛКМ/ПКМ). Сохраняется в конфиг.
**Язык интерфейса:** ПКМ → «language» — переключает En/Ru (с рестартом).

## Конфигурация

Файл `~/.config/raskladka/config` создаётся автоматически при первом
переназначении клавиши или смене языка через меню трея. Пример:

```
trigger=ShiftLeft
timeout_ms=400
lang=en
```

- `trigger` — имя варианта из
  [Key](https://docs.rs/rdev/0.5.3/rdev/enum.Key.html) (`ControlLeft`,
  `Alt`, `F1`...) или `m:Middle` для средней кнопки мыши
- `timeout_ms` — таймаут двойного нажатия (по умолчанию 400)
- `lang` — язык интерфейса: `en` или `ru`

## Ссылки

- [Releases](https://github.com/kiberdans/raskladka/releases)

## Лицензия

MIT
