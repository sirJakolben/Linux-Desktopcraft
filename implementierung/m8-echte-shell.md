# M8: Echte Shell

**Planungsstand:** grob, nicht bereit.
- **Teil 1 vorgezogen** direkt nach M0 A5 (Nutzer 18.10.2026), konkretisiert (18.10.2026): Abschnitt „Teil 1“. Bereit, sobald S1 (Prototyp P15) besteht.
- Recherche fehlt: R30.
- Fragen an den Nutzer fehlen: F11 (Shell-Grundfunktionen).
- Hängt von allen vorigen Meilensteinen ab.

**Größe:** 3–8 Wochen, Risiko hoch.
**Voraussetzung:** Teil 1: M0 bis A5. Rest: M3 und M4.
**Features:** [features.md](../features.md) §1 (PC startet in der Welt), §8 (System). **Belege:** recherche-ergebnisse.md R5, R6, R14.

## Teil 1: Login-Sitzung mit Testszene

Ziel: die m0-Testszene als eigene Sitzung, im Alltag benutzbar (Terminal, Firefox, X11-Programme). Code: Crate `nested` aus `prototypen/` mit zweitem Backend.

**Entscheidungen** (Nutzer 18.10.2026):
- Nur der Hauptmonitor; der zweite bleibt schwarz. Mehrere Monitore: R30.
- Programmstart über Tastenkürzel (Terminal foot) und Autostart-Liste.
- X11-Programme über `xwayland-satellite` (prüft P10).

**Empfehlung, durch S1 zu bestätigen:** Ausgabe über die wgpu-DRM-Surface (`SurfaceTargetUnsafe::Drm`, Vulkan `VK_EXT_acquire_drm_display` + `VK_KHR_display`; beide meldet der Treiber 615). Rückfall: GBM-Puffer von Smithay, wgpu rendert in den importierten dmabuf, vor dem Page-Flip auf die GPU warten, Ausgabe über Smithays `DrmSurface`.

**Hardware:** beide Monitore an der NVIDIA-Karte (`card2`, DP-2 und DP-3); `nvidia_drm` `modeset=Y`, `fbdev=Y`. Sitzungsverwaltung über logind (seatd nicht installiert, nicht nötig).

**Platzhalter** (Bedienung später in bedienung.md):
- Super+Enter: foot. Strg+Alt+Rücktaste: Shell beenden (Notaus). Strg+Alt+F1–F12: Konsole wechseln.
- Mauszeiger und Fadenkreuz zeichnet die Shell selbst (kein Host-Cursor mehr).
- Mehr Screens: 6 im Halbkreis um den Startpunkt, neues Fenster auf den nächsten freien (D9 offen).
- Autostart-Liste als Textdatei, Format mit S4 festlegen (T15 offen).

**Entwicklungsweg:** erst auf einer freien Konsole (Strg+Alt+F3, anmelden, Binary starten, beenden → zurück mit Strg+Alt+F2 zu GNOME), erst mit S5 als Eintrag im Anmeldebildschirm.

| Nr. | Inhalt | Abnahme (ohne Codelesen) |
| --- | ------ | ------------------------ |
| S1 | Prototyp P15: Testszene über wgpu-DRM-Surface auf dem Hauptmonitor, von einer freien Konsole gestartet | Szene auf dem Monitor (Foto oder Screenshot per Taste), Beenden führt zurück zur Konsole, GNOME danach intakt; sonst Rückfall wählen |
| S2 | Backend-Wahl `--drm` neben winit; libseat (logind) und libinput: Maus, Tastatur, Konsolenwechsel, Notaus | Testskripte aus m0 (A4, A5) laufen auch im DRM-Backend; Konsolenwechsel hin und zurück ohne schwarzes Bild |
| S3 | Selbst gezeichneter Zeiger und Fadenkreuz; 6 Screens | Screenshot mit Zeiger über einem Fenster; 6 Fenster gleichzeitig |
| S4 | Super+Enter, Autostart-Liste, `xwayland-satellite` mit `DISPLAY` für Clients | foot per Kürzel; Firefox per Autostart; `xeyes` sichtbar und klickbar (P10) |
| S5 | Sitzung: Wrapper-Skript `~/Scripts/minecraft-shell-sitzung.sh` (Umgebung: `XDG_CURRENT_DESKTOP`, `XDG_SESSION_TYPE=wayland`, `dbus-update-activation-environment`, Log nach `~/.local/state`), `.desktop` per E2 | Eintrag „Minecraft Shell“ im Anmeldebildschirm; provozierter Absturz führt zurück zum Anmeldebildschirm, GNOME-Sitzung startet normal |
| S6 | Abnahme | eine Stunde Alltag (Terminal, Firefox, ein X11-Programm) ohne Absturz; danach Performance-Durchgang |

**E2** (Nutzer, root, nach S5): Sitzungsdatei `/usr/share/wayland-sessions/minecraft-shell.desktop` installieren; genauer Befehl kommt mit S5.

**Risiken:**
- `VK_EXT_acquire_drm_display` auf NVIDIA 615 ungetestet; S1 entscheidet. Konsolenwechsel mit einer Vulkan-Display-Surface (DRM-Master abgeben) ist der heikelste Teil von S2.
- Ohne Fehlerbild kein Rückweg auf der Konsole: Notaus und Absturz müssen die Konsole freigeben (Smithay-Session/libseat). Immer Strg+Alt+F2 (GNOME) als Ausweg testen.
- Smithay-Features für S2: `backend_session_libseat`, `backend_libinput`, `backend_udev` (zusätzlich zu m0).

## Ziel

Die Shell wird zur eigenen Login-Sitzung mit den Systemfunktionen aus §8.

## Kern: darauf kommt es an

1. **Notfallmodus, bevor die Shell Standard wird.** Das ist entschieden. Zunächst über die Sitzung: die GNOME-Sitzung (T4 in m0) bleibt als zweite Sitzung im Display-Manager. Optional wechselt ein Wrapper bei einem Exit-Code ungleich 0 automatisch dorthin (Entwurfsidee, ungetestet).
2. **Absturzverhalten:** Stürzt der Compositor ab, sterben alle Programme mit. Nur Qt-6-Programme können sich wieder verbinden. Deshalb darf ein Fehler im Weltrenderer den Compositor nicht beenden (T3), und offene Arbeit ist in Gefahr.
3. **Echte Hardware statt nested:**
   - DRM/KMS, libinput, libseat
   - Ausgabe der wgpu-Bilder auf DRM (M0, P15)
   - Direct Scanout
   - mehrere Monitore

## Zu entscheiden

- **Welche Shell-Grundfunktionen** in welcher Form: F11.
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
- **NVIDIA** (Treiber und Explicit Sync: m0): Cursor-Planes mit linearem Modifier. Bekannt: Aufwachen aus dem Ruhezustand hängt mit Kernel 7.x auf Blackwell (Umgehung `s2idle`).

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
