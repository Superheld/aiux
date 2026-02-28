# AIUX

> Ein neuronales, LLM-gesteuertes Betriebssystem.
> Embodied AI - ein OS das wahrnimmt, versteht und handelt.

## Was ist AIUX?

AIUX ist ein minimales Linux-System auf Basis von Alpine Linux, in dem KI keine
App ist, sondern eine Schicht des Betriebssystems. Mensch und LLM arbeiten
kooperativ - das System denkt mit, nimmt wahr und handelt.

Siehe [docs/PRD.md](docs/PRD.md) für die vollständige Vision.

## Hardware

- Raspberry Pi 4 (8 GB RAM)
- microSD-Karte (mind. 8 GB)
- Ethernet-Kabel oder WiFi
- HDMI-Monitor + Tastatur (nur für Ersteinrichtung)

## Manuelle Installation

> Später wird `build/build-image.sh` ein fertiges Image erzeugen.
> Bis dahin: manuelle Einrichtung.

### 1. Alpine Linux herunterladen

```bash
curl -LO https://dl-cdn.alpinelinux.org/alpine/v3.23/releases/aarch64/alpine-rpi-3.23.3-aarch64.tar.gz
```

### 2. SD-Karte vorbereiten

SD-Karte identifizieren (z.B. `/dev/mmcblk0` oder `/dev/sdX`):

```bash
lsblk
```

Partitionieren und formatieren:

```bash
# Zwei Partitionen: 512 MB Boot (FAT32) + Rest Daten (ext4)
sudo sfdisk --wipe always /dev/mmcblk0 <<EOF
start=2048, size=512M, type=c, bootable
start=1050624, type=83
EOF

sudo mkfs.vfat -n ALPINEPI -F 32 /dev/mmcblk0p1
sudo mkfs.ext4 -L AIUXDATA /dev/mmcblk0p2
```

> Hinweis: `dosfstools` muss installiert sein fuer `mkfs.vfat`.

### 3. Alpine auf die Boot-Partition

```bash
sudo mkdir -p /mnt/sdcard
sudo mount /dev/mmcblk0p1 /mnt/sdcard
sudo tar xzf alpine-rpi-3.23.3-aarch64.tar.gz -C /mnt/sdcard/
sudo umount /mnt/sdcard
```

### 4. Erster Boot

- SD-Karte in den Raspi
- HDMI-Monitor und Tastatur anschliessen
- Strom anschliessen - der Pi bootet
- Login: `root` (kein Passwort)

### 5. Alpine Grundkonfiguration

```bash
setup-alpine
```

Empfohlene Einstellungen:

| Option | Wert |
|--------|------|
| Keyboard layout | `de` |
| Hostname | `aiux` |
| Interface | `eth0` (oder `wlan0`) |
| IP | `dhcp` |
| Root password | sicheres Passwort setzen |
| User anlegen | ja (wird der LLM-User) |
| Timezone | `Europe/Berlin` |
| NTP client | `chrony` |
| Mirror | `f` (fastest) |
| Disk | `none` (diskless mode) |

### 6. SSH einrichten

SSH-Root-Login aktivieren (für Ersteinrichtung):

```bash
apk add openssh
rc-update add sshd
rc-service sshd start
echo "PermitRootLogin yes" >> /etc/ssh/sshd_config
rc-service sshd restart
```

IP-Adresse herausfinden:

```bash
ip addr show eth0 | grep inet
```

Vom Hauptrechner verbinden:

```bash
ssh root@<raspi-ip>
```

### 7. SSH-Key einrichten

Auf dem Hauptrechner (falls noch kein Key vorhanden):

```bash
ssh-keygen -t ed25519
```

Key auf den Raspi kopieren:

```bash
ssh-copy-id root@<raspi-ip>
```

Danach Root-Login mit Passwort wieder deaktivieren:

```bash
sed -i 's/^PermitRootLogin yes/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
rc-service sshd restart
```

### 8. Basis-Firewall

```bash
apk add iptables ip6tables

# Nur SSH erlauben, Rest blockieren
iptables -A INPUT -i lo -j ACCEPT
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -A INPUT -p tcp --dport 22 -j ACCEPT
iptables -A INPUT -j DROP

# Regeln speichern
rc-service iptables save
rc-update add iptables
```

### 9. Konfiguration sichern

Alpine im diskless mode verliert Änderungen beim Reboot. Sichern mit:

```bash
lbu commit -d
```

> Wichtig: Nach jeder Konfigurationsänderung `lbu commit -d` ausfuehren!

## Projektstruktur

```
aiux/
├── docs/           # PRD, Roadmap
├── build/          # Image-Build-System
├── agent/          # aiux-agent (Rust) - LLM Co-Pilot
├── nerve/          # aiux-nerve (Rust) - Lokale neuronale Netze
├── hub/            # aiux-hub (Rust) - Verbindet alles
└── scripts/        # Phase-1 Shell-Skripte
```

## Tech-Stack

- **OS**: Alpine Linux (musl, busybox)
- **Sprache**: Rust
- **LLM**: Anthropic Claude, Mistral (API-basiert)
- **Lokale Inference**: ONNX Runtime
- **IPC**: Unix Sockets

## Lizenz

MIT - siehe [LICENSE](LICENSE)
