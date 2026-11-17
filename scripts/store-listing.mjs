#!/usr/bin/env node
/**
 * Fills the Microsoft Store listing template exported from Partner Center.
 *
 *   node scripts/store-listing.mjs
 *
 * Every listing language has its own column in the template. This script keeps
 * the CSV in sync with the copy below: shared fields (title, publisher, images)
 * are repeated in each column, while the marketing text, features, requirements
 * and search terms are localized per language.
 *
 * The script is idempotent: it rewrites the same fields on the existing
 * template and keeps the UTF-8 BOM and CRLF line endings Partner Center expects.
 */

import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const DIR = 'docs/store-listing';
const template = readdirSync(DIR).find((file) => /^listingData-.*\.csv$/.test(file));
if (!template) throw new Error('No listingData-*.csv template found in docs/store-listing');

/**
 * Listing columns, in the order Partner Center exported them plus the extra
 * languages. Each column points at one entry of COPY.
 */
const LANGUAGES = [
  { column: 'fr-fr', copy: 'fr' },
  { column: 'en-us', copy: 'en' },
  { column: 'de-de', copy: 'de' },
  { column: 'es-es', copy: 'es' },
  { column: 'it-it', copy: 'it' },
];

/** Sort and voice titles for the Store (name plus a keyword, per the Partner Center SEO guidance). */
const SORT_TITLES = {
  en: 'Dexorium Offline Pokédex',
  fr: 'Dexorium Pokédex hors ligne',
  de: 'Dexorium Offline-Pokédex',
  es: 'Dexorium Pokédex sin conexión',
  it: 'Dexorium Pokédex offline',
};

const VOICE_TITLES = {
  en: 'Dexorium Pokémon encyclopedia',
  fr: 'Dexorium encyclopédie Pokémon',
  de: 'Dexorium Pokémon Lexikon',
  es: 'Dexorium enciclopedia Pokémon',
  it: 'Dexorium enciclopedia Pokémon',
};

const SCREENSHOT_FILES = [
  'screenshot-1-pokedex-dark-1920x1080.png',
  'screenshot-2-species-dark-1920x1080.png',
  'screenshot-3-type-matrix-light-1920x1080.png',
  'screenshot-4-moves-dark-1920x1080.png',
  'screenshot-5-abilities-dark-1920x1080.png',
  'screenshot-6-home-dark-1920x1080.png',
  'screenshot-7-settings-dark-1920x1080.png',
];

const IMAGES = {
  StoreLogo300x300: 'store-listing/images/logos/StoreLogo-300x300.png',
  StoreLogo720x1080: 'store-listing/images/logos/StoreLogo-720x1080.png',
  StoreLogo1080x1080: 'store-listing/images/logos/StoreLogo-1080x1080.png',
  PromoImage1920x1080: 'store-listing/images/promo/promo-1920x1080.png',
  PromoImage2400x1200: 'store-listing/images/promo/promo-2400x1200.png',
  OptionalPromo358x358: 'store-listing/images/promo/promo-358x358.png',
  OptionalPromo1000x800: 'store-listing/images/promo/promo-1000x800.png',
  OptionalPromo414x180: 'store-listing/images/promo/promo-414x180.png',
};

const COPY = {
  en: {
    description: `Dexorium is a fast, fully offline Pokémon encyclopedia: a Pokédex for players, collectors and competitive trainers that keeps working with no connection.

No account, no ads, no telemetry. Every screen works in a plane, at a tournament table or in a room without Wi-Fi.

WHAT IS INSIDE
• Pokédex — 1,025 species with types, base stats, abilities, measurements, breeding data, evolution chains, forms and learnsets.
• Moves — 919 moves with type, category, power, accuracy, PP and generation, plus filters to narrow them down.
• Abilities — 374 abilities with their in-game descriptions.
• Type chart — the complete type-effectiveness matrix, from generation 1 to generation 9.
• Search — jump to any species, move or ability with Ctrl+K.
• Favourites — keep the species you care about close at hand.

BUILT FOR OFFLINE USE
The whole reference database ships inside the app: nothing is downloaded at runtime, and your favourites stay on your device.

DESIGNED FOR WINDOWS
A native app built with Tauri and Rust, with instant startup, light and dark themes, English and French interfaces, and a compact layout that stays readable at 1366 × 768.

Dexorium is an unofficial, fan-made reference tool. It is not affiliated with, endorsed by or sponsored by Nintendo, Creatures Inc. or The Pokémon Company. Pokémon and Pokémon character names are trademarks of their respective owners.`,
    releaseNotes: `First Microsoft Store release.

• Offline Pokédex with 1,025 species
• 919 moves and 374 abilities with filters
• Complete type chart for generations 1 to 9
• Instant search (Ctrl+K) and local favourites
• Light and dark themes
• English and French interfaces
• No account, no ads, no telemetry`,
    shortDescription:
      'Offline Pokémon encyclopedia: 1,025 species, 919 moves, 374 abilities and the full type chart. No account, no ads, no telemetry.',
    copyright:
      'Copyright © 2026 Martzcode. Pokémon and Pokémon character names are trademarks of their respective owners. Dexorium is an unofficial fan project, not affiliated with Nintendo, Creatures Inc. or The Pokémon Company.',
    license:
      'Dexorium is free software distributed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later). Source code and license text: https://github.com/Martzcode/Dexorium',
    features: [
      '1,025 species with types, base stats, abilities, breeding data, evolution chains and learnsets',
      '919 moves with filters by type, category, generation and power',
      '374 abilities with their in-game descriptions',
      'Complete type-effectiveness chart for generations 1 to 9',
      'Instant global search with Ctrl+K',
      'Favourites stored locally on your device',
      'Light and dark themes',
      'English and French interfaces',
      'Works fully offline: no account, no ads, no telemetry',
      'Native Windows app built with Tauri and Rust: fast startup, small footprint',
    ],
    captions: [
      'Browse 1,025 species with instant search',
      'Stats, abilities, breeding data and evolutions',
      'The complete type chart for generations 1 to 9',
      'Filter 919 moves by type, category and generation',
      '374 abilities with in-game descriptions',
      'Every Pokémon, fully offline',
      'Dataset details, themes and languages',
    ],
    minimum: [
      'OS: Windows 10 version 2004 (build 19041) or later',
      'Architecture: x64',
      'Processor: 1 GHz or faster, 2 cores',
      'Memory: 4 GB RAM',
      'Storage: 250 MB available space',
      'Display: 1366 × 768 or higher',
      'Graphics: DirectX 9 compatible',
      'Microsoft Edge WebView2 Runtime (installed by the Microsoft Store)',
    ],
    recommended: [
      'OS: Windows 11',
      'Memory: 8 GB RAM',
      'Storage: SSD with 500 MB available',
      'Display: 1920 × 1080 or higher',
    ],
    searchTerms: [
      'pokedex',
      'pokemon pokedex offline',
      'pokemon encyclopedia',
      'pokemon type chart',
      'pokemon moves list',
      'pokemon abilities',
      'pokemon database',
    ],
  },

  fr: {
    description: `Dexorium est une encyclopédie Pokémon rapide et entièrement hors ligne : un Pokédex pour les joueurs, les collectionneurs et les dresseurs compétitifs qui continue de fonctionner sans aucune connexion.

Sans compte, sans publicité, sans télémétrie. Chaque écran fonctionne en avion, à une table de tournoi ou dans une salle sans Wi-Fi.

CE QUE CONTIENT L'APPLICATION
• Pokédex — 1 025 espèces avec types, statistiques de base, talents, mensurations, données de reproduction, chaînes d'évolution, formes et ensembles d'attaques apprises.
• Attaques — 919 attaques avec type, catégorie, puissance, précision, PP et génération, plus des filtres pour affiner la liste.
• Talents — 374 talents avec leur description en jeu.
• Table des types — la matrice d'efficacité complète, de la génération 1 à la génération 9.
• Recherche — accédez à n'importe quelle espèce, attaque ou talent avec Ctrl+K.
• Favoris — gardez les espèces qui vous intéressent à portée de main.

CONÇUE POUR LE HORS LIGNE
Toute la base de référence est embarquée dans l'application : rien n'est téléchargé au démarrage et vos favoris restent sur votre appareil.

PENSÉE POUR WINDOWS
Une application native construite avec Tauri et Rust : démarrage instantané, thèmes clair et sombre, interface en français et en anglais, et une mise en page compacte qui reste lisible en 1366 × 768.

Dexorium est un outil de référence non officiel, réalisé par des fans. Il n'est ni affilié, ni approuvé, ni sponsorisé par Nintendo, Creatures Inc. ou The Pokémon Company. Pokémon et les noms des personnages Pokémon sont des marques de leurs propriétaires respectifs.`,
    releaseNotes: `Première version sur le Microsoft Store.

• Pokédex hors ligne avec 1 025 espèces
• 919 attaques et 374 talents avec filtres
• Table des types complète pour les générations 1 à 9
• Recherche instantanée (Ctrl+K) et favoris locaux
• Thèmes clair et sombre
• Interface en français et en anglais
• Sans compte, sans publicité, sans télémétrie`,
    shortDescription:
      'Encyclopédie Pokémon hors ligne : 1 025 espèces, 919 attaques, 374 talents et la table des types complète. Sans compte, sans publicité, sans télémétrie.',
    copyright:
      'Copyright © 2026 Martzcode. Pokémon et les noms des personnages Pokémon sont des marques de leurs propriétaires respectifs. Dexorium est un projet de fans non officiel, non affilié à Nintendo, Creatures Inc. ou The Pokémon Company.',
    license:
      'Dexorium est un logiciel libre distribué sous licence GNU Affero General Public License v3.0 ou ultérieure (AGPL-3.0-or-later). Code source et texte de la licence : https://github.com/Martzcode/Dexorium',
    features: [
      '1 025 espèces avec types, statistiques de base, talents, données de reproduction, chaînes d’évolution et attaques apprises',
      '919 attaques avec filtres par type, catégorie, génération et puissance',
      '374 talents avec leur description en jeu',
      'Table d’efficacité des types complète, de la génération 1 à la génération 9',
      'Recherche globale instantanée avec Ctrl+K',
      'Favoris enregistrés localement sur votre appareil',
      'Thèmes clair et sombre',
      'Interface en français et en anglais',
      'Entièrement hors ligne : sans compte, sans publicité, sans télémétrie',
      'Application Windows native construite avec Tauri et Rust : démarrage rapide, empreinte réduite',
    ],
    captions: [
      'Parcourez 1 025 espèces avec la recherche instantanée',
      'Statistiques, talents, reproduction et évolutions',
      'La table des types complète, de la génération 1 à 9',
      'Filtrez 919 attaques par type, catégorie et génération',
      '374 talents avec leur description en jeu',
      'Tous les Pokémon, entièrement hors ligne',
      'Détails du jeu de données, thèmes et langues',
    ],
    minimum: [
      'Système : Windows 10 version 2004 (build 19041) ou ultérieur',
      'Architecture : x64',
      'Processeur : 1 GHz ou plus, 2 cœurs',
      'Mémoire : 4 Go de RAM',
      'Stockage : 250 Mo d’espace disponible',
      'Écran : 1366 × 768 ou supérieur',
      'Graphiques : compatible DirectX 9',
      'Microsoft Edge WebView2 Runtime (installé par le Microsoft Store)',
    ],
    recommended: [
      'Système : Windows 11',
      'Mémoire : 8 Go de RAM',
      'Stockage : SSD avec 500 Mo disponibles',
      'Écran : 1920 × 1080 ou supérieur',
    ],
    searchTerms: [
      'pokedex',
      'pokedex pokemon hors ligne',
      'encyclopedie pokemon',
      'table des types pokemon',
      'liste des attaques pokemon',
      'talents pokemon',
      'base de donnees pokemon',
    ],
  },

  de: {
    description: `Dexorium ist ein schnelles, vollständig offline nutzbares Pokémon-Lexikon: ein Pokédex für Spieler, Sammler und kompetitive Trainer, der ohne Verbindung weiter funktioniert.

Kein Konto, keine Werbung, keine Telemetrie. Jeder Bildschirm funktioniert im Flugzeug, am Turniertisch oder in einem Raum ohne WLAN.

DAS IST ENTHALTEN
• Pokédex — 1.025 Arten mit Typen, Basiswerten, Fähigkeiten, Maßen, Zuchtdaten, Entwicklungsreihen, Formen und erlernten Attacken.
• Attacken — 919 Attacken mit Typ, Kategorie, Stärke, Genauigkeit, AP und Generation, dazu Filter zum Eingrenzen.
• Fähigkeiten — 374 Fähigkeiten mit ihrer Beschreibung aus dem Spiel.
• Typentabelle — die vollständige Effektivitätsmatrix von Generation 1 bis Generation 9.
• Suche — mit Strg+K direkt zu jeder Art, Attacke oder Fähigkeit springen.
• Favoriten — die Arten, die dich interessieren, immer griffbereit.

FÜR DEN OFFLINE-EINSATZ GEBAUT
Die gesamte Referenzdatenbank ist in der App enthalten: Zur Laufzeit wird nichts heruntergeladen, und deine Favoriten bleiben auf deinem Gerät.

FÜR WINDOWS GEMACHT
Eine native App mit Tauri und Rust: sofortiger Start, helles und dunkles Design, Oberfläche auf Englisch und Französisch sowie ein kompaktes Layout, das auch bei 1366 × 768 lesbar bleibt.

Dexorium ist ein inoffizielles Fan-Projekt. Es steht in keiner Verbindung zu Nintendo, Creatures Inc. oder The Pokémon Company und wird von diesen weder unterstützt noch gesponsert. Pokémon und die Namen der Pokémon-Charaktere sind Marken ihrer jeweiligen Inhaber.`,
    releaseNotes: `Erste Veröffentlichung im Microsoft Store.

• Offline-Pokédex mit 1.025 Arten
• 919 Attacken und 374 Fähigkeiten mit Filtern
• Vollständige Typentabelle für die Generationen 1 bis 9
• Sofortige Suche (Strg+K) und lokale Favoriten
• Helles und dunkles Design
• Oberfläche auf Englisch und Französisch
• Kein Konto, keine Werbung, keine Telemetrie`,
    shortDescription:
      'Offline-Pokémon-Lexikon: 1.025 Arten, 919 Attacken, 374 Fähigkeiten und die komplette Typentabelle. Ohne Konto, ohne Werbung, ohne Telemetrie.',
    copyright:
      'Copyright © 2026 Martzcode. Pokémon und die Namen der Pokémon-Charaktere sind Marken ihrer jeweiligen Inhaber. Dexorium ist ein inoffizielles Fan-Projekt und steht in keiner Verbindung zu Nintendo, Creatures Inc. oder The Pokémon Company.',
    license:
      'Dexorium ist freie Software, veröffentlicht unter der GNU Affero General Public License v3.0 oder neuer (AGPL-3.0-or-later). Quellcode und Lizenztext: https://github.com/Martzcode/Dexorium',
    features: [
      '1.025 Arten mit Typen, Basiswerten, Fähigkeiten, Zuchtdaten, Entwicklungsreihen und erlernten Attacken',
      '919 Attacken mit Filtern nach Typ, Kategorie, Generation und Stärke',
      '374 Fähigkeiten mit ihrer Beschreibung aus dem Spiel',
      'Vollständige Typentabelle für die Generationen 1 bis 9',
      'Sofortige globale Suche mit Strg+K',
      'Favoriten, lokal auf deinem Gerät gespeichert',
      'Helles und dunkles Design',
      'Oberfläche auf Englisch und Französisch',
      'Vollständig offline: kein Konto, keine Werbung, keine Telemetrie',
      'Native Windows-App mit Tauri und Rust: schneller Start, geringer Speicherbedarf',
    ],
    captions: [
      '1.025 Arten mit sofortiger Suche durchsuchen',
      'Werte, Fähigkeiten, Zuchtdaten und Entwicklungen',
      'Die vollständige Typentabelle für Generation 1 bis 9',
      '919 Attacken nach Typ, Kategorie und Generation filtern',
      '374 Fähigkeiten mit ihrer Beschreibung aus dem Spiel',
      'Alle Pokémon, vollständig offline',
      'Dataset-Details, Designs und Sprachen',
    ],
    minimum: [
      'Betriebssystem: Windows 10 Version 2004 (Build 19041) oder neuer',
      'Architektur: x64',
      'Prozessor: 1 GHz oder schneller, 2 Kerne',
      'Arbeitsspeicher: 4 GB RAM',
      'Speicherplatz: 250 MB verfügbar',
      'Bildschirm: 1366 × 768 oder höher',
      'Grafik: DirectX 9-kompatibel',
      'Microsoft Edge WebView2 Runtime (wird vom Microsoft Store installiert)',
    ],
    recommended: [
      'Betriebssystem: Windows 11',
      'Arbeitsspeicher: 8 GB RAM',
      'Speicherplatz: SSD mit 500 MB verfügbar',
      'Bildschirm: 1920 × 1080 oder höher',
    ],
    searchTerms: [
      'pokedex',
      'pokemon pokedex offline',
      'pokemon lexikon',
      'pokemon typentabelle',
      'pokemon attacken liste',
      'pokemon fahigkeiten',
      'pokemon datenbank',
    ],
  },

  es: {
    description: `Dexorium es una enciclopedia Pokémon rápida y totalmente sin conexión: una Pokédex para jugadores, coleccionistas y entrenadores competitivos que sigue funcionando sin conexión.

Sin cuenta, sin anuncios, sin telemetría. Todas las pantallas funcionan en un avión, en una mesa de torneo o en una sala sin wifi.

QUÉ INCLUYE
• Pokédex — 1025 especies con tipos, estadísticas base, habilidades, medidas, datos de crianza, cadenas evolutivas, formas y movimientos aprendidos.
• Movimientos — 919 movimientos con tipo, categoría, potencia, precisión, PP y generación, además de filtros para acotar la lista.
• Habilidades — 374 habilidades con su descripción en el juego.
• Tabla de tipos — la matriz de eficacia completa, de la generación 1 a la generación 9.
• Búsqueda — accede a cualquier especie, movimiento o habilidad con Ctrl+K.
• Favoritos — ten a mano las especies que te interesan.

DISEÑADA PARA USARSE SIN CONEXIÓN
Toda la base de referencia viene incluida en la app: no se descarga nada en tiempo de ejecución y tus favoritos se quedan en tu dispositivo.

PENSADA PARA WINDOWS
Una app nativa creada con Tauri y Rust: inicio inmediato, temas claro y oscuro, interfaz en inglés y francés, y un diseño compacto que sigue siendo legible a 1366 × 768.

Dexorium es una herramienta de referencia no oficial, creada por fanes. No está afiliada, respaldada ni patrocinada por Nintendo, Creatures Inc. ni The Pokémon Company. Pokémon y los nombres de los personajes Pokémon son marcas de sus respectivos propietarios.`,
    releaseNotes: `Primera versión en Microsoft Store.

• Pokédex sin conexión con 1025 especies
• 919 movimientos y 374 habilidades con filtros
• Tabla de tipos completa para las generaciones 1 a 9
• Búsqueda instantánea (Ctrl+K) y favoritos locales
• Temas claro y oscuro
• Interfaz en inglés y francés
• Sin cuenta, sin anuncios, sin telemetría`,
    shortDescription:
      'Enciclopedia Pokémon sin conexión: 1025 especies, 919 movimientos, 374 habilidades y la tabla de tipos completa. Sin cuenta, sin anuncios, sin telemetría.',
    copyright:
      'Copyright © 2026 Martzcode. Pokémon y los nombres de los personajes Pokémon son marcas de sus respectivos propietarios. Dexorium es un proyecto de fanes no oficial, no afiliado a Nintendo, Creatures Inc. ni The Pokémon Company.',
    license:
      'Dexorium es software libre distribuido bajo la GNU Affero General Public License v3.0 o posterior (AGPL-3.0-or-later). Código fuente y texto de la licencia: https://github.com/Martzcode/Dexorium',
    features: [
      '1025 especies con tipos, estadísticas base, habilidades, datos de crianza, cadenas evolutivas y movimientos aprendidos',
      '919 movimientos con filtros por tipo, categoría, generación y potencia',
      '374 habilidades con su descripción en el juego',
      'Tabla de eficacia de tipos completa para las generaciones 1 a 9',
      'Búsqueda global instantánea con Ctrl+K',
      'Favoritos guardados localmente en tu dispositivo',
      'Temas claro y oscuro',
      'Interfaz en inglés y francés',
      'Funciona totalmente sin conexión: sin cuenta, sin anuncios, sin telemetría',
      'App nativa para Windows creada con Tauri y Rust: inicio rápido y poco espacio',
    ],
    captions: [
      'Explora 1025 especies con búsqueda instantánea',
      'Estadísticas, habilidades, crianza y evoluciones',
      'La tabla de tipos completa, de la generación 1 a la 9',
      'Filtra 919 movimientos por tipo, categoría y generación',
      '374 habilidades con su descripción en el juego',
      'Todos los Pokémon, totalmente sin conexión',
      'Detalles del conjunto de datos, temas e idiomas',
    ],
    minimum: [
      'Sistema operativo: Windows 10 versión 2004 (compilación 19041) o posterior',
      'Arquitectura: x64',
      'Procesador: 1 GHz o superior, 2 núcleos',
      'Memoria: 4 GB de RAM',
      'Almacenamiento: 250 MB de espacio disponible',
      'Pantalla: 1366 × 768 o superior',
      'Gráficos: compatible con DirectX 9',
      'Microsoft Edge WebView2 Runtime (instalado por Microsoft Store)',
    ],
    recommended: [
      'Sistema operativo: Windows 11',
      'Memoria: 8 GB de RAM',
      'Almacenamiento: SSD con 500 MB disponibles',
      'Pantalla: 1920 × 1080 o superior',
    ],
    searchTerms: [
      'pokedex',
      'pokedex pokemon sin conexion',
      'enciclopedia pokemon',
      'tabla de tipos pokemon',
      'lista de movimientos pokemon',
      'habilidades pokemon',
      'base de datos pokemon',
    ],
  },

  it: {
    description: `Dexorium è un'enciclopedia Pokémon veloce e completamente offline: un Pokédex per giocatori, collezionisti e allenatori competitivi che continua a funzionare senza connessione.

Nessun account, nessuna pubblicità, nessuna telemetria. Ogni schermata funziona in aereo, al tavolo di un torneo o in una stanza senza Wi-Fi.

COSA CONTIENE
• Pokédex — 1.025 specie con tipi, statistiche base, abilità, misure, dati di allevamento, catene evolutive, forme e mosse apprese.
• Mosse — 919 mosse con tipo, categoria, potenza, precisione, PP e generazione, oltre a filtri per restringere la lista.
• Abilità — 374 abilità con la loro descrizione in gioco.
• Tabella dei tipi — la matrice di efficacia completa, dalla generazione 1 alla generazione 9.
• Ricerca — passa a qualsiasi specie, mossa o abilità con Ctrl+K.
• Preferiti — tieni a portata di mano le specie che ti interessano.

PENSATA PER L'USO OFFLINE
L'intero database di riferimento è incluso nell'app: nulla viene scaricato durante l'esecuzione e i preferiti restano sul tuo dispositivo.

PROGETTATA PER WINDOWS
Un'app nativa realizzata con Tauri e Rust: avvio immediato, temi chiaro e scuro, interfaccia in inglese e francese e un layout compatto che resta leggibile a 1366 × 768.

Dexorium è uno strumento di riferimento non ufficiale, realizzato da fan. Non è affiliato, approvato o sponsorizzato da Nintendo, Creatures Inc. o The Pokémon Company. Pokémon e i nomi dei personaggi Pokémon sono marchi dei rispettivi proprietari.`,
    releaseNotes: `Prima versione su Microsoft Store.

• Pokédex offline con 1.025 specie
• 919 mosse e 374 abilità con filtri
• Tabella dei tipi completa per le generazioni da 1 a 9
• Ricerca istantanea (Ctrl+K) e preferiti locali
• Temi chiaro e scuro
• Interfaccia in inglese e francese
• Nessun account, nessuna pubblicità, nessuna telemetria`,
    shortDescription:
      'Enciclopedia Pokémon offline: 1.025 specie, 919 mosse, 374 abilità e la tabella dei tipi completa. Nessun account, nessuna pubblicità, nessuna telemetria.',
    copyright:
      'Copyright © 2026 Martzcode. Pokémon e i nomi dei personaggi Pokémon sono marchi dei rispettivi proprietari. Dexorium è un progetto amatoriale non ufficiale, non affiliato a Nintendo, Creatures Inc. o The Pokémon Company.',
    license:
      'Dexorium è software libero distribuito con licenza GNU Affero General Public License v3.0 o successiva (AGPL-3.0-or-later). Codice sorgente e testo della licenza: https://github.com/Martzcode/Dexorium',
    features: [
      '1.025 specie con tipi, statistiche base, abilità, dati di allevamento, catene evolutive e mosse apprese',
      '919 mosse con filtri per tipo, categoria, generazione e potenza',
      '374 abilità con la loro descrizione in gioco',
      'Tabella di efficacia dei tipi completa per le generazioni da 1 a 9',
      'Ricerca globale istantanea con Ctrl+K',
      'Preferiti salvati localmente sul tuo dispositivo',
      'Temi chiaro e scuro',
      'Interfaccia in inglese e francese',
      'Funziona completamente offline: nessun account, nessuna pubblicità, nessuna telemetria',
      'App nativa per Windows realizzata con Tauri e Rust: avvio rapido e ingombro ridotto',
    ],
    captions: [
      'Sfoglia 1.025 specie con ricerca istantanea',
      'Statistiche, abilità, allevamento ed evoluzioni',
      'La tabella dei tipi completa, dalla generazione 1 alla 9',
      'Filtra 919 mosse per tipo, categoria e generazione',
      '374 abilità con la loro descrizione in gioco',
      'Tutti i Pokémon, completamente offline',
      'Dettagli del dataset, temi e lingue',
    ],
    minimum: [
      'Sistema operativo: Windows 10 versione 2004 (build 19041) o successiva',
      'Architettura: x64',
      'Processore: 1 GHz o superiore, 2 core',
      'Memoria: 4 GB di RAM',
      'Spazio di archiviazione: 250 MB disponibili',
      'Schermo: 1366 × 768 o superiore',
      'Grafica: compatibile con DirectX 9',
      'Microsoft Edge WebView2 Runtime (installato da Microsoft Store)',
    ],
    recommended: [
      'Sistema operativo: Windows 11',
      'Memoria: 8 GB di RAM',
      'Spazio di archiviazione: SSD con 500 MB disponibili',
      'Schermo: 1920 × 1080 o superiore',
    ],
    searchTerms: [
      'pokedex',
      'pokedex pokemon offline',
      'enciclopedia pokemon',
      'tabella dei tipi pokemon',
      'elenco mosse pokemon',
      'abilità pokemon',
      'database pokemon',
    ],
  },
};

/** Field values for one listing language, keyed by the CSV field name. */
function valuesFor(copy, code) {
  const values = {
    Description: copy.description,
    ReleaseNotes: copy.releaseNotes,
    ShortDescription: copy.shortDescription,
    CopyrightTrademarkInformation: copy.copyright,
    AdditionalLicenseTerms: copy.license,
    // Fields that are identical in every language.
    Title: 'Dexorium',
    DevStudio: 'Martzcode',
    OverrideLogosForWin10: 'False',
    SortTitle: SORT_TITLES[code],
    VoiceTitle: VOICE_TITLES[code],
    ...IMAGES,
  };

  SCREENSHOT_FILES.forEach((file, index) => {
    values[`DesktopScreenshot${index + 1}`] = `store-listing/images/screenshots/${file}`;
    values[`DesktopScreenshotCaption${index + 1}`] = copy.captions[index];
  });

  copy.features.forEach((feature, index) => {
    values[`Feature${index + 1}`] = feature;
  });

  copy.minimum.forEach((requirement, index) => {
    values[`MinimumHardwareReq${index + 1}`] = requirement;
  });

  copy.recommended.forEach((requirement, index) => {
    values[`RecommendedHardwareReq${index + 1}`] = requirement;
  });

  copy.searchTerms.forEach((term, index) => {
    values[`SearchTerm${index + 1}`] = term;
  });

  // French and Italian typesetting uses the typographic apostrophe. Search
  // terms are excluded from the healthy queries users actually type.
  if (code === 'fr' || code === 'it') {
    for (const [field, value] of Object.entries(values)) {
      if (!field.startsWith('SearchTerm')) {
        values[field] = value.replace(/(\p{L})'(\p{L})/gu, '$1\u2019$2');
      }
    }
  }

  return values;
}

/** Guards against a language missing a bullet and silently leaving a column empty. */
function assertSameShape() {
  const reference = COPY.en;
  for (const [code, copy] of Object.entries(COPY)) {
    for (const key of ['features', 'captions', 'minimum', 'recommended', 'searchTerms']) {
      if (copy[key].length !== reference[key].length) {
        throw new Error(
          `${code}.${key} has ${copy[key].length} entries, expected ${reference[key].length}`,
        );
      }
    }
  }
}

/** Splits RFC 4180 CSV text into rows, keeping quoted newlines and commas. */
function parseCsv(text) {
  const rows = [];
  let row = [];
  let cell = '';
  let quoted = false;

  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];

    if (quoted) {
      if (char === '"') {
        if (text[index + 1] === '"') {
          cell += '"';
          index += 1;
        } else {
          quoted = false;
        }
      } else {
        cell += char;
      }
      continue;
    }

    if (char === '"') {
      quoted = true;
    } else if (char === ',') {
      row.push(cell);
      cell = '';
    } else if (char === '\r' || char === '\n') {
      row.push(cell);
      rows.push(row);
      row = [];
      cell = '';
      if (char === '\r' && text[index + 1] === '\n') index += 1;
    } else {
      cell += char;
    }
  }

  if (cell !== '' || row.length > 0) {
    row.push(cell);
    rows.push(row);
  }

  return rows;
}

const escape = (value) => {
  const normalized = value.replace(/\r?\n/g, '\n');
  return /[",\n]/.test(normalized) ? `"${normalized.replaceAll('"', '""')}"` : normalized;
};

assertSameShape();

const source = readFileSync(join(DIR, template), 'utf8').replace(/^\uFEFF/, '');
const rows = parseCsv(source);
const header = rows[0];

for (const { column } of LANGUAGES) {
  if (!header.includes(column)) header.push(column);
}

const byLanguage = Object.fromEntries(
  LANGUAGES.map(({ column, copy }) => [column, valuesFor(COPY[copy], copy)]),
);

const seen = new Set();
const unknown = new Set();

const output = rows.map((cells, index) => {
  if (index === 0) return header;

  const field = cells[0];
  const row = header.map((_, columnIndex) => cells[columnIndex] ?? '');

  if (!field) return row;

  const firstLanguage = byLanguage[LANGUAGES[0].column];
  if (!(field in firstLanguage)) {
    unknown.add(field);
    return row;
  }

  for (const { column } of LANGUAGES) {
    const value = byLanguage[column][field];
    row[header.indexOf(column)] = value;
  }
  seen.add(field);
  return row;
});

const missing = Object.keys(byLanguage[LANGUAGES[0].column]).filter((field) => !seen.has(field));
if (missing.length) {
  throw new Error(`Fields missing from the template: ${missing.join(', ')}`);
}

const csv = output.map((cells) => cells.map(escape).join(',')).join('\n');

writeFileSync(join(DIR, template), '\uFEFF' + csv, 'utf8');

console.log(`Filled ${seen.size} fields for ${LANGUAGES.length} languages in ${template}`);
if (unknown.size) {
  console.log(`Left ${unknown.size} template-only fields untouched`);
}
