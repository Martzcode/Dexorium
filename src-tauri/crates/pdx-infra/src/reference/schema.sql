-- Reference database schema (fixture subset).
--
-- This mirrors the tables documented in
-- `docs/pokedex/pokedex-pro-specifications.md` (section 3.5.1) but only keeps
-- the columns required by the first vertical slice. Columns documented for
-- later features (abilities, moves, encounters, evolution chains, media) are
-- added by the future `pdx-dataset` migrations.

PRAGMA foreign_keys = ON;

-- =========================================================
-- RÉFÉRENTIELS
-- =========================================================
CREATE TABLE generation (
    id          INTEGER PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    region_slug TEXT NOT NULL
);

CREATE TABLE poke_type (
    id            INTEGER PRIMARY KEY,
    slug          TEXT NOT NULL UNIQUE,
    introduced_in INTEGER NOT NULL REFERENCES generation(id)
);

-- =========================================================
-- ESPÈCES ET FORMES
-- =========================================================
CREATE TABLE species (
    id                  INTEGER PRIMARY KEY,
    slug                TEXT NOT NULL UNIQUE,
    national_dex_number INTEGER NOT NULL UNIQUE,
    generation_id       INTEGER NOT NULL REFERENCES generation(id),
    capture_rate        INTEGER NOT NULL,
    base_happiness      INTEGER,
    is_legendary        INTEGER NOT NULL DEFAULT 0,
    is_mythical         INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE pokemon_form (
    id          INTEGER PRIMARY KEY,
    species_id  INTEGER NOT NULL REFERENCES species(id) ON DELETE CASCADE,
    slug        TEXT NOT NULL UNIQUE,
    form_order  INTEGER NOT NULL,
    is_default  INTEGER NOT NULL DEFAULT 0,
    height_dm   INTEGER NOT NULL,
    weight_hg   INTEGER NOT NULL
);
CREATE INDEX idx_form_species ON pokemon_form(species_id);

CREATE TABLE form_type (
    form_id INTEGER NOT NULL REFERENCES pokemon_form(id) ON DELETE CASCADE,
    slot    INTEGER NOT NULL CHECK (slot IN (1, 2)),
    type_id INTEGER NOT NULL REFERENCES poke_type(id),
    PRIMARY KEY (form_id, slot)
) WITHOUT ROWID;
CREATE INDEX idx_form_type_lookup ON form_type(type_id, form_id);

CREATE TABLE form_stat_summary (
    form_id INTEGER PRIMARY KEY REFERENCES pokemon_form(id) ON DELETE CASCADE,
    hp      INTEGER NOT NULL,
    attack  INTEGER NOT NULL,
    defense INTEGER NOT NULL,
    sp_atk  INTEGER NOT NULL,
    sp_def  INTEGER NOT NULL,
    speed   INTEGER NOT NULL,
    bst     INTEGER NOT NULL
);
CREATE INDEX idx_stat_bst ON form_stat_summary(bst);
CREATE INDEX idx_stat_speed ON form_stat_summary(speed);

-- =========================================================
-- LOCALISATION
-- =========================================================
CREATE TABLE translation (
    entity_type TEXT NOT NULL,   -- species, form, move, ability, item, type, location...
    entity_id   INTEGER NOT NULL,
    lang        TEXT NOT NULL,   -- fr, en, ja, de, es, it, ko, zh-Hans
    field       TEXT NOT NULL,   -- name, genus, effect_short, description...
    value       TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id, lang, field)
) WITHOUT ROWID;
CREATE INDEX idx_trans_lookup ON translation(entity_type, lang, field);

-- =========================================================
-- RECHERCHE PLEIN TEXTE
-- =========================================================
CREATE VIRTUAL TABLE search_index USING fts5(
    entity_type     UNINDEXED,
    entity_id       UNINDEXED,
    name,
    name_normalized,   -- sans accents, minuscules, sans ponctuation
    aliases,
    tokenize = 'unicode61 remove_diacritics 2'
);

-- =========================================================
-- MÉTADONNÉES DU DATASET
-- =========================================================
CREATE TABLE dataset_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
