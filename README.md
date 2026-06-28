# raskladka

Переключение раскладки клавиатуры (QWERTY ↔ ЙЦУКЕН) по двойному нажатию Shift.

## Установка

### Зависимости

- **X11:** `xclip`, `xdotool`
- **Wayland:** `wl-clipboard`, `wtype`

```bash
# Arch
sudo pacman -S xclip xdotool wl-clipboard wtype

# Ubuntu/Debian
sudo apt install xclip xdotool wl-clipboard wtype

# Fedora
sudo dnf install xclip xdotool wl-clipboard wtype
```

### Сборка

```bash
git clone https://github.com/kiberdans/raskladka
cd raskladka
cargo build --release
sudo ./target/release/raskladka
```

### Автозагрузка (systemd)

```bash
cp raskladka.service ~/.config/systemd/user/
systemctl --user enable --now raskladka.service
```

### Права доступа

`rdev` требует доступ к `/dev/input/*`. Добавьте себя в группу `input`:

```bash
sudo usermod -aG input $USER
# перелогиньтесь
```

Либо запускайте с `sudo`.

## Использование

1. Выделите текст
2. Дважды быстро нажмите Shift
3. Раскладка переключится

**Трей:** ЛКМ — вкл/выкл, ПКМ — меню, наведение — тултип.

## Лицензия

MIT
