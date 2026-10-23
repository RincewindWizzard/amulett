# Bewegungssensor: QMI8658

6-Achsen-IMU (Beschleunigung + Gyroskop), I2C-Adresse `0x6B`, Interrupts über
TCA9554 P6 (INT1) und GPIO21 (INT2). Treiber: `src/imu.rs`.

## Konfiguration im PoC

| Wert | Einstellung | Empfindlichkeit |
|---|---|---|
| Beschleunigung | ±4 g, 125 Hz | 8192 LSB/g |
| Gyroskop | ±512 °/s, 125 Hz | 64 LSB/(°/s) |

```rust
--8<-- "src/imu.rs:init"
```

`CTRL1 = 0x40` aktiviert das automatische Hochzählen der Registeradresse,
damit alle zwölf Datenbytes in einem Zugriff gelesen werden können.

## Messwerte lesen

```rust
--8<-- "src/imu.rs:sample"
```

Liegt das Board flach und ruhig, zeigt die Z-Achse etwa **1 g** und das
Gyroskop Werte nahe null. Die Seite **IMU** zeigt die Werte als Zahlen und die
X/Y-Neigung als Balken.

## Stolpersteine

- Die Chip-Kennung (`WHO_AM_I`, Register `0x00`) muss `0x05` sein.
- Die Datenreihenfolge ist Beschleunigung X/Y/Z, dann Gyroskop X/Y/Z, jeweils
  Little-Endian.
- Das Datenblatt ist als "Advance Information" gekennzeichnet.

## Herstellerunterlagen

- [QMI8658-Datenblatt](https://www.waveshare.net/w/upload/5/5f/QMI8658C.pdf)
- [Beispiel `04_LVGL_QMI8658_ui`](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/tree/main/examples/arduino/04_LVGL_QMI8658_ui)
- [Registerdefinitionen in SensorLib](https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75/blob/main/examples/arduino/libraries/SensorLib/src/REG/QMI8658Constants.h)
