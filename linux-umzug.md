# Minecraft Shell – Umzug auf Linux

Stand: 16.10.2026 · Status: **Phase A (Vorbereitung unter Windows) steht an**

> **Hinweis für KI-Sessions:** Schritt-für-Schritt-Anleitung für den Umstieg des Nutzers auf Linux. Sie ist die Vorbereitung vor Meilenstein 0 (siehe [M0](implementierung/m0-entscheidungen-und-prototypen.md)). Erledigte Schritte abhaken. Schritte, die Systemeinstellungen ändern, führt der Nutzer selbst aus; Claude erklärt und prüft.

## Ausgangslage (ausgelesen am 16.10.2026)

| Punkt                       | Befund                                                                                                                                                  | Folge                                                                                                                                     |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| SSD                         | WD_BLACK SN850X 2 TB („Datenträger 1“). Seit A4: C: 1362 GB (375 GB frei), dahinter **500 GB „Nicht zugeordnet“**, danach Wiederherstellung (0,94 GB)   | Linux kommt in den nicht zugeordneten Bereich                                                                                             |
| HDD                         | Toshiba 1 TB, D:, enthält nur eine kleine `SteamLibrary`                                                                                                | bleibt vorerst Datenplatte                                                                                                                |
| EFI-Partition von Windows   | nur **100 MB**                                                                                                                                          | zu klein für Fedora (Mindestgröße 500 MB, [FESCo #2994](https://pagure.io/fesco/issue/2994)). Linux bekommt eine **eigene EFI-Partition** |
| Verschlüsselung (BitLocker) | **aus** (`manage-bde -status C:`: „Vollständig entschlüsselt“, keine Schlüsselschutzvorrichtungen; Geräteverschlüsselung auf diesem PC nicht verfügbar) | kein Wiederherstellungsschlüssel nötig                                                                                                    |
| Schnellstart                | **aus** (seit 16.10.2026, `HiberbootEnabled` = 0); Ruhezustand selbst ist noch verfügbar                                                                | Windows-Partition für Linux nicht mehr gesperrt, sobald Windows einmal ganz heruntergefahren oder neu gestartet wurde                     |
| Secure Boot                 | **aus**                                                                                                                                                 | so lassen, das NVIDIA-Treiber-Setup wird dadurch einfacher                                                                                |
| Sonst                       | i7-12700K, 64 GB RAM, RTX 5060 Ti 16 GB                                                                                                                 | reicht für alles                                                                                                                          |

## Geplante Aufteilung der SSD

| Bereich                                 | Größe                            | Inhalt                                                                             |
| --------------------------------------- | -------------------------------- | ---------------------------------------------------------------------------------- |
| Windows-EFI, MSR, C:, Wiederherstellung | wie bisher, C: um 500 GB kleiner | Windows, bleibt unangetastet                                                       |
| **Linux-EFI** (neu)                     | 1 GB                             | Bootloader von Fedora                                                              |
| **/boot** (neu)                         | 2 GB                             | Kernel                                                                             |
| **Btrfs** (neu)                         | Rest (~497 GB)                   | System und persönliche Dateien (`/` und `/home` als Subvolumes, Snapshots möglich) |

- [x] Phase A: Vorbereitung unter Windows (jetzt) 
## Phase B: Installation

Begleitung durch eine KI-Session: [prompts/linux-umzug-begleiter.md](prompts/linux-umzug-begleiter.md).

- [x] **B0** Vor dem ersten Neustart: Anleitung für Phase B als Webseite fürs Handy anlegen (während der Installation läuft kein Claude). Erledigt: [Handy-Anleitung](https://claude.ai/artifact/2vbv2jY9FJ5q1cgGMAfoA4), Quelle [prompts/fedora-installation-handy.html](prompts/fedora-installation-handy.html). Wichtigster Befund: Fedora 44 KDE nutzt den neuen Web-Installer; Partitionen im „storage editor“ anlegen, Btrfs mit Unterbereichen `root` und `home` (sonst Abbruch, Bug 2463653). Projektdateien: entschieden am 17.10.2026 über **GitHub**, öffentliches Repository [sirJakolben/Linux-Desktopcraft](https://github.com/sirJakolben/Linux-Desktopcraft).
- [ ] **B1** Vom Stick starten (Boot-Menü mit **F12**, Mainboard Gigabyte B760 GAMING X AX) und Fedora zuerst nur ausprobieren (Live-System): Bild, WLAN/LAN, Tastatur. Bleibt der Bildschirm schwarz, im Startmenü „Basic Graphics Mode" wählen; der richtige NVIDIA-Treiber kommt erst in Phase C.
- [ ] **B2** Installieren mit **manueller Partitionierung**, nur im freien Bereich: Linux-EFI 1 GB (Einhängepunkt `/boot/efi`, EFI-Systempartition), `/boot` 2 GB (ext4), Rest Btrfs für `/` und `/home`. **Keine automatische Partitionierung**: Sie würde die 100-MB-EFI-Partition von Windows benutzen und scheitern. Windows-Partitionen nicht anfassen. (Den genauen Ablauf im Installer gehen wir zusammen durch, wenn es so weit ist.)
- [ ] **B3** Neu starten: Im Fedora-Bootmenü muss auch „Windows Boot Manager" stehen. Windows einmal starten und prüfen, ob alles läuft.

## Phase C: Einrichtung unter Linux

- [ ] **C0** Claude Code installieren, das Repository klonen (`git clone https://github.com/sirJakolben/Linux-Desktopcraft.git`) und den Begleiter-Prompt in einer neuen Session unter Linux erneut einsetzen.
- [ ] **C1** System aktualisieren: `sudo dnf upgrade --refresh`, dann neu starten. Danach auf Fedora 45 upgraden (erscheint planmäßig am 20.10.2026), vor C2, damit der NVIDIA-Treiber gleich für 45 gebaut wird.
- [ ] **C2** NVIDIA-Treiber: RPM-Fusion-Paketquellen aktivieren, `akmod-nvidia` (Production-Zweig 595, offenes Modul) installieren, warten, bis das Modul gebaut ist (einige Minuten), neu starten, mit `nvidia-smi` prüfen.
- [ ] **C3** Rollback: Snapper für Btrfs-Snapshots einrichten, damit ein kaputtes Update rückgängig gemacht werden kann.
- [ ] **C4** Uhrzeit: Windows und Linux lesen die Hardware-Uhr unterschiedlich. Damit sie nicht springt, Windows auf UTC umstellen (Registry-Eintrag `RealTimeIsUniversal`).
- [ ] **C5** Windows-Partition und HDD unter Linux einbinden (lesen und schreiben), damit Dateien während des Umzugs erreichbar sind.

## Phase D: Umzug der Arbeit

- [ ] **D1** Git, SSH-Schlüssel bzw. GitHub CLI (`gh`) einrichten.
- [ ] **D2** Prüfen, ob es die Claude-Desktop-App mit Code-Tab für Linux gibt (Claude Code selbst ist seit C0 da). Hinweis vom 17.10.2026: Die Claude-Code-Doku verlinkt eine Linux-Version der Desktop-App ([code.claude.com/docs/en/desktop-linux](https://code.claude.com/docs/en/desktop-linux)), noch nicht geprüft.
- [ ] **D3** IntelliJ IDEA (über die JetBrains Toolbox), passendes JDK für die Minecraft-Version von Smart Villagers, Gradle läuft über das Projekt.
- [ ] **D4** Smart Villagers klonen, bauen und den Test-Client starten (`runClient`).
- [ ] **D5** Prism Launcher installieren, Minecraft-Instanzen, Mods und Welten übernehmen. Spielen testen.
- [ ] **D6** Projekt Minecraft Shell übernehmen (klonen oder von der Windows-Partition kopieren).

## Phase E: Windows entfernen (später, wenn alles unter Linux läuft)

Das ist möglich und unkompliziert, **weil Linux seine eigene EFI-Partition hat**:

1. Prüfen, dass nichts mehr auf C: gebraucht wird (Phase D abgeschlossen, Daten gesichert).
2. Unter Linux mit der KDE-Partitionsverwaltung die Windows-Partitionen der SSD löschen: Windows-EFI (100 MB), MSR, C:, Wiederherstellung. Die **Linux-EFI-Partition bleibt**.
3. Den Starteintrag „Windows Boot Manager" entfernen (`efibootmgr`) und das Bootmenü neu erzeugen.
4. Den frei gewordenen Platz nutzen: Er liegt *vor* der Linux-Partition, deshalb nicht verschieben, sondern als zusätzliches Gerät ins Btrfs aufnehmen (`btrfs device add`) oder als eigene Partition einbinden (z. B. für Spiele).
5. Optional die HDD neu formatieren (Btrfs oder ext4), vorher die `SteamLibrary` sichern.
