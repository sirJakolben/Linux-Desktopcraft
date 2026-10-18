# M8: Echte Shell

**Planungsstand:** grob, nicht bereit.
- **Teil 1 vorgezogen** direkt nach M0 A5 (Nutzer 18.10.2026), konkretisiert (18.10.2026): Abschnitt „Teil 1“. S1–S2 bereit; der Ausgabeweg gilt erst nach P15 (S3 in der VM, S5 auf NVIDIA) als entschieden.
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

**Empfehlung, durch P15 zu bestätigen:** Ausgabe über die wgpu-DRM-Surface (`SurfaceTargetUnsafe::Drm`, Vulkan `VK_EXT_acquire_drm_display` + `VK_KHR_display`; beide meldet der Treiber 615). Rückfall: GBM-Puffer von Smithay, wgpu rendert in den importierten dmabuf, vor dem Page-Flip auf die GPU warten, Ausgabe über Smithays `DrmSurface`.

**Hardware:** beide Monitore an der NVIDIA-Karte (`card2`, DP-2 und DP-3); `nvidia_drm` `modeset=Y`, `fbdev=Y`. Sitzungsverwaltung über logind (seatd nicht installiert, nicht nötig).

**Platzhalter** (Bedienung später in bedienung.md):
- Super+Enter: foot. Strg+Alt+Rücktaste: Shell beenden (Notaus). Strg+Alt+F1–F12: Konsole wechseln.
- Mauszeiger und Fadenkreuz zeichnet die Shell selbst (kein Host-Cursor mehr).
- Mehr Screens: 6 im Halbkreis um den Startpunkt, neues Fenster auf den nächsten freien (D9 offen).
- Autostart-Liste als Textdatei, Format mit S4 festlegen (T15 offen).

**Ablauf** nach der Testreihenfolge (implementierungsplan.md Abschnitt 3): S1–S2 nested, S3–S4 in der VM, S5 auf einer freien Konsole, S6 als echte Sitzung. Nested gilt statt Super die konfigurierbare Modifier-Taste (GNOME fängt Super ab).

| Nr. | Phase | Inhalt | Abnahme (ohne Codelesen) |
| --- | ----- | ------ | ------------------------ |
| S1 | nested | Selbst gezeichneter Zeiger und Fadenkreuz; 6 Screens | Screenshot mit Zeiger über einem Fenster; 6 Fenster gleichzeitig |
| S2 | nested | Tastenkürzel für foot, Autostart-Liste, `xwayland-satellite` mit `DISPLAY` für Clients | foot per Kürzel; Firefox per Autostart; `xeyes` sichtbar und klickbar (P10) |
| S3 | VM | Prototyp P15 im Gast; Backend-Wahl `--drm` neben winit; libseat (logind) und libinput: Maus, Tastatur, Konsolenwechsel, Notaus | Testszene im VM-Fenster (Screenshot); Testskripte A4, A5 laufen im DRM-Backend; Konsolenwechsel hin und zurück; Notaus führt zur Gast-Konsole |
| S4 | VM | Sitzung: Wrapper-Skript `~/Scripts/minecraft-shell-sitzung.sh` (Umgebung: `XDG_CURRENT_DESKTOP`, `XDG_SESSION_TYPE=wayland`, `dbus-update-activation-environment`, Log nach `~/.local/state`), `.desktop` im Gast | Eintrag „Minecraft Shell“ im Anmeldebildschirm des Gasts; provozierter Absturz führt dorthin zurück |
| S5 | Konsole | P15 auf der NVIDIA-Karte: dasselbe Binary von Strg+Alt+F3 auf dem Hauptmonitor; zurück zu GNOME mit Strg+Alt+F2 | Szene auf dem Monitor (Foto oder Screenshot per Taste); A4, A5 laufen; Konsolenwechsel ohne schwarzes Bild; Beenden führt zur Konsole; GNOME danach intakt; sonst Rückfall wählen |
| S6 | Generaltest | Sitzung auf dem echten System (E2) | Eintrag im Anmeldebildschirm; Absturz führt dorthin zurück; GNOME-Sitzung startet normal; eine Stunde Alltag (Terminal, Firefox, ein X11-Programm) ohne Absturz; danach Performance-Durchgang |

**E3** (Nutzer, root, vor S3): QEMU/KVM und virt-manager installieren, Fedora-Workstation-Gast einrichten, Ordner `prototypen/` per virtiofs freigeben; genauer Befehl kommt mit S3.
**E2** (Nutzer, root, vor S6): Sitzungsdatei `/usr/share/wayland-sessions/minecraft-shell.desktop` installieren; genauer Befehl kommt mit S4.

**Offen, klärt S3:** Vulkan im Gast (virtio-gpu; lavapipe als Software oder Venus) und ob dort `VK_KHR_display` geht. Empfehlung: Geht es nicht, im Gast in einen Dumb-Buffer kopieren (nur VM, Performance egal).

**Risiken:**
- `VK_EXT_acquire_drm_display` auf NVIDIA 615 ungetestet; erst S5 entscheidet, die VM prüft es nicht. Konsolenwechsel mit einer Vulkan-Display-Surface (DRM-Master abgeben) ist der heikelste Teil.
- Ohne Fehlerbild kein Rückweg auf der Konsole: Notaus und Absturz müssen die Konsole freigeben (Smithay-Session/libseat). In S5 immer Strg+Alt+F2 (GNOME) als Ausweg testen.
- Smithay-Features für S3: `backend_session_libseat`, `backend_libinput`, `backend_udev` (zusätzlich zu m0).

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
