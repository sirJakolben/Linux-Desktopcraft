# M9: Multiplayer-Sessions

**Planungsstand:** grob, nicht bereit. Bewusst der letzte Schritt, nach dem Release.
**Größe:** nicht abschätzbar, Risiko sehr hoch.
**Features:** [features.md](../features.md) §10.

## Was frühere Meilensteine schon beachten sollen

- **Weltzustand und Layout serialisierbar halten:** Positionen, Screens und Waypoints liegen in klaren Datenstrukturen, nicht verstreut im Renderer.
- **Nur Lesen bleibt Pflicht:** Auch gemeinsame Sitzungen schreiben nie in Weltdateien.
- **Vorbild:** WebDisplays überträgt Klicks als Vektor auf der Blockseite an alle Clients in der Nähe.

## Offene Fragen

Alles. Erst nach dem Release mit dem Nutzer klären.
