# M9: Multiplayer-Sessions

**Planungsstand:** grob, nicht bereit. Bewusst der letzte Schritt, nach dem Release.
**Größe laut Featureliste:** nicht abschätzbar, Risiko sehr hoch.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M9 bekannt ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md).

## Ziel

Mehrere Nutzer sind in derselben Minecraft-Welt. Man kann zu anderen hinlaufen und die Fenster ansehen und öffnen, die sie oft geöffnet haben.

## Kern

Dafür braucht es Netzwerk, Synchronisierung und ein Rechtekonzept: Wer sieht und bedient welche Fenster? Das ist noch nicht ausgearbeitet.

## Was frühere Meilensteine schon beachten sollen

- **Weltzustand und Layout serialisierbar halten:** Positionen, Screens und Waypoints liegen in klaren Datenstrukturen, nicht verstreut im Renderer.
- **Nur Lesen bleibt Pflicht:** Auch gemeinsame Sitzungen schreiben nie in Weltdateien.
- **Vorbild:** WebDisplays überträgt Klicks als Vektor auf der Blockseite an alle Clients in der Nähe.

## Offene Fragen

Alles. Erst nach dem Release mit dem Nutzer klären.
