# M8: Echte Shell

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R30.
- Fragen an den Nutzer fehlen: Featureliste, Frage 11 (Shell-Grundfunktionen).
- Hängt von allen vorigen Meilensteinen ab.

**Größe laut Featureliste:** 3–8 Wochen, Risiko hoch.
**Voraussetzung:** M3 und M4.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M8 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md), Quellen in [recherche-ergebnisse.md](recherche-ergebnisse.md), Abschnitt R5/R6/R14.

## Ziel

Die Shell wird zur eigenen Login-Sitzung: Der PC startet direkt in die Welt.

Dazu gehören:
- Benachrichtigungen
- Audio
- mehrere Monitore
- Energiesparen
- Absturzschutz mit Notfallmodus

## Kern: darauf kommt es an

1. **Notfallmodus, bevor die Shell Standard wird.** Das ist entschieden. Zunächst über die Sitzung: Plasma bleibt als zweite Sitzung im Display-Manager. Optional wechselt ein Wrapper bei einem Exit-Code ungleich 0 automatisch dorthin (Entwurfsidee, ungetestet).
2. **Absturzverhalten:** Stürzt der Compositor ab, sterben alle Programme mit. Nur Qt-6-Programme können sich wieder verbinden. Deshalb darf ein Fehler im Weltrenderer den Compositor nicht beenden (T3), und offene Arbeit ist in Gefahr.
3. **Echte Hardware statt nested:**
   - DRM/KMS, libinput, libseat
   - Ausgabe der wgpu-Bilder auf DRM (M0, P15)
   - Direct Scanout
   - mehrere Monitore

## Entschieden

- Vollständige Desktop-Shell; der PC startet direkt in der Welt.
- Der Notfallmodus muss existieren, bevor die Shell Standard wird.

## Zu entscheiden

- **Welche Shell-Grundfunktionen** in welcher Form (Featureliste, Frage 11):
  - Sperrbildschirm, Abmelden und Herunterfahren
  - Tastaturlayout
  - Lautstärke und Helligkeit
  - Netzwerk und Bluetooth
  - System-Tray
  - Bildschirmfreigabe und Screenshots
- **Mehrere Monitore:** Was zeigt jeder Monitor?
- **Prozessaufteilung:** Wird die Welt in einen eigenen Prozess ausgelagert (T3)? Erst entscheiden, wenn sich Abstürze in der Praxis zeigen.

## Technische Grundlagen

- **Sitzung:**
  - `.desktop` in `/usr/share/wayland-sessions/`
  - systemd-User-Units (Vorbild niri: `niri.service`, `niri-shutdown.target`, `--session` importiert die Umgebung)
  - alternativ uwsm
- **Portale:**
  - eigene `<desktop>-portals.conf`
  - Dateidialoge über `xdg-desktop-portal-gtk`
  - Bildschirmfreigabe braucht Capture-Protokolle im Compositor (`ext-image-copy-capture`, für `xdg-desktop-portal-wlr`)
- **Benachrichtigungen:** D-Bus-Dienst `org.freedesktop.Notifications` selbst anbieten.
- **Wiederverbindung nach Absturz:** `QT_WAYLAND_RECONNECT=1`, `wl-restart` als Muster für einen Socket, der einen Neustart überlebt.
- **NVIDIA:** Explicit Sync, Production-Zweig 595, Cursor-Planes mit linearem Modifier. Bekannt: Aufwachen aus dem Ruhezustand hängt mit Kernel 7.x auf Blackwell (Umgehung `s2idle`).

## Unbedingt beachten

- **Nie ohne Rückweg:** Auch wenn die Shell nicht startet, muss der Nutzer an einen normalen Desktop kommen.
- **Lücken der Shell** sind für den Alltag Pflicht, nicht optional. Ohne Sperrbildschirm und Bildschirmfreigabe ist die Shell nicht alltagstauglich.
- **Energiesparen:** Im Leerlauf keine Bilder (T10). Den Ruhezustand mit NVIDIA testen.

## Abnahme

Konkretisieren, wenn der Meilenstein geplant wird. Mindestens:
- Eine Woche Alltag als Standardsitzung.
- Ein provozierter Absturz führt zurück zu einem benutzbaren Desktop.

## Referenzen

- [niri Getting Started](https://github.com/YaLTeR/niri/wiki/Getting-Started) (Sitzung, systemd, Portale; nur die Technik)
- [portals.conf](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html)
- [wl-restart](https://github.com/ferdi265/wl-restart)
- [river 0.4](https://isaacfreund.com/blog/river-window-management/) (Fenstermanager in einem eigenen Prozess)
