-- Reference database schema.
--
-- Mirrors `docs/pokedex/pokedex-pro-specifications.md` (section 3.5.1) for the
-- scope of the MVP. Items, locations and encounters are intentionally omitted:
-- they belong to phase 2 and are not part of the MVP perimeter.
--
-- The schema is shared by two consumers:
--   * `pdx-dataset` builds the real reference database from the pinned sources;
--   * `SqliteSpeciesRepository::from_fixture` seeds a small in-memory database
--     for tests and for the browser fallback.
--
-- Columns that the fixture does not provide keep a default so both consumers
-- can execute the exact same script.

PRAGMA foreign_keys = ON;

-- =========================================================
-- RÉFÉRENTIELS
-- =========================================================
CREATE TABLE generation (
    id          INTEGER PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    region_slug TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE version_group (
    id            INTEGER PRIMARY KEY,
    slug          TEXT NOT NULL UNIQUE,
    generation_id INTEGER NOT NULL REFERENCES generation(id),
    order_index   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE game_version (
    id               INTEGER PRIMARY KEY,
    slug             TEXT NOT NULL UNIQUE,
    version_group_id INTEGER NOT NULL REFERENCES version_group(id)
);

CREATE TABLE poke_type (
    id            INTEGER PRIMARY KEY,
    slug          TEXT NOT NULL UNIQUE,
    introduced_in INTEGER NOT NULL REFERENCES generation(id)
);

-- Type effectiveness matrix, versioned by generation.
CREATE TABLE type_efficacy (
    generation_id   INTEGER NOT NULL REFERENCES generation(id),
    attacking_type  INTEGER NOT NULL REFERENCES poke_type(id),
    defending_type  INTEGER NOT NULL REFERENCES poke_type(id),
    multiplier_x100 INTEGER NOT NULL,
    PRIMARY KEY (generation_id, attacking_type, defending_type)
) WITHOUT ROWID;

CREATE TABLE egg_group (
    id   INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE
);

-- =========================================================
-- ESPÈCES ET FORMES
-- =========================================================
CREATE TABLE evolution_chain (
    id                     INTEGER PRIMARY KEY,
    baby_trigger_item_slug TEXT
);

CREATE TABLE species (
    id                      INTEGER PRIMARY KEY,
    slug                    TEXT NOT NULL UNIQUE,
    national_dex_number     INTEGER NOT NULL UNIQUE,
    generation_id           INTEGER NOT NULL REFERENCES generation(id),
    gender_rate             INTEGER NOT NULL DEFAULT -1,   -- -1 = asexué, sinon /8
    capture_rate            INTEGER NOT NULL DEFAULT 0,
    base_happiness          INTEGER,
    hatch_counter           INTEGER,
    growth_rate_slug        TEXT NOT NULL DEFAULT 'medium',
    color_slug              TEXT,
    shape_slug              TEXT,
    habitat_slug            TEXT,
    is_baby                 INTEGER NOT NULL DEFAULT 0,
    is_legendary            INTEGER NOT NULL DEFAULT 0,
    is_mythical             INTEGER NOT NULL DEFAULT 0,
    is_ultra_beast          INTEGER NOT NULL DEFAULT 0,
    has_gender_differences  INTEGER NOT NULL DEFAULT 0,
    evolution_chain_id      INTEGER REFERENCES evolution_chain(id),
    evolves_from_species_id INTEGER REFERENCES species(id)
);
CREATE INDEX idx_species_generation ON species(generation_id);

CREATE TABLE species_egg_group (
    species_id   INTEGER NOT NULL REFERENCES species(id) ON DELETE CASCADE,
    egg_group_id INTEGER NOT NULL REFERENCES egg_group(id),
    PRIMARY KEY (species_id, egg_group_id)
) WITHOUT ROWID;

-- A playable form: Charizard, Mega Charizard X, Alolan Rattata...
CREATE TABLE pokemon_form (
    id                      INTEGER PRIMARY KEY,
    species_id              INTEGER NOT NULL REFERENCES species(id) ON DELETE CASCADE,
    slug                    TEXT NOT NULL UNIQUE,
    form_order              INTEGER NOT NULL DEFAULT 1,
    is_default              INTEGER NOT NULL DEFAULT 0,
    is_mega                 INTEGER NOT NULL DEFAULT 0,
    is_gigantamax           INTEGER NOT NULL DEFAULT 0,
    is_regional             INTEGER NOT NULL DEFAULT 0,
    region_slug             TEXT,
    introduced_generation_id INTEGER NOT NULL DEFAULT 1 REFERENCES generation(id),
    height_dm               INTEGER NOT NULL DEFAULT 0,
    weight_hg               INTEGER NOT NULL DEFAULT 0,
    base_experience         INTEGER
);
CREATE INDEX idx_form_species ON pokemon_form(species_id);

CREATE TABLE form_type (
    form_id INTEGER NOT NULL REFERENCES pokemon_form(id) ON DELETE CASCADE,
    slot    INTEGER NOT NULL CHECK (slot IN (1, 2)),
    type_id INTEGER NOT NULL REFERENCES poke_type(id),
    PRIMARY KEY (form_id, slot)
) WITHOUT ROWID;
CREATE INDEX idx_form_type_lookup ON form_type(type_id, form_id);

CREATE TABLE form_stat (
    form_id      INTEGER NOT NULL REFERENCES pokemon_form(id) ON DELETE CASCADE,
    stat_slug    TEXT NOT NULL,   -- hp, attack, defense, sp_atk, sp_def, speed
    base_value   INTEGER NOT NULL,
    effort_yield INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (form_id, stat_slug)
) WITHOUT ROWID;

-- Denormalized copy used by list sorting and filtering (avoids 6 joins).
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
-- TALENTS
-- =========================================================
CREATE TABLE ability (
    id            INTEGER PRIMARY KEY,
    slug          TEXT NOT NULL UNIQUE,
    generation_id INTEGER NOT NULL REFERENCES generation(id),
    is_main_series INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE form_ability (
    form_id    INTEGER NOT NULL REFERENCES pokemon_form(id) ON DELETE CASCADE,
    ability_id INTEGER NOT NULL REFERENCES ability(id),
    slot       INTEGER NOT NULL,     -- 1, 2, 3 (3 = hidden)
    is_hidden  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (form_id, slot)
) WITHOUT ROWID;
CREATE INDEX idx_ability_forms ON form_ability(ability_id);

-- =========================================================
-- ATTAQUES
-- =========================================================
CREATE TABLE move_ (
    id              INTEGER PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE,
    generation_id   INTEGER NOT NULL REFERENCES generation(id),
    type_id         INTEGER NOT NULL REFERENCES poke_type(id),
    damage_class    TEXT NOT NULL CHECK (damage_class IN ('physical', 'special', 'status')),
    power           INTEGER,
    accuracy        INTEGER,
    pp              INTEGER,
    priority        INTEGER NOT NULL DEFAULT 0,
    target_slug     TEXT NOT NULL,
    effect_chance   INTEGER,
    crit_rate_bonus INTEGER NOT NULL DEFAULT 0,
    drain_pct       INTEGER NOT NULL DEFAULT 0,
    healing_pct     INTEGER NOT NULL DEFAULT 0,
    min_hits        INTEGER,
    max_hits        INTEGER,
    min_turns       INTEGER,
    max_turns       INTEGER,
    ailment_slug    TEXT,
    ailment_chance  INTEGER,
    flinch_chance   INTEGER NOT NULL DEFAULT 0,
    stat_chance     INTEGER NOT NULL DEFAULT 0,
    flags_bitmask   INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_move_type ON move_(type_id);
CREATE INDEX idx_move_class ON move_(damage_class);
CREATE INDEX idx_move_power ON move_(power);

CREATE TABLE move_past_value (
    move_id          INTEGER NOT NULL REFERENCES move_(id) ON DELETE CASCADE,
    version_group_id INTEGER NOT NULL REFERENCES version_group(id),
    power            INTEGER,
    accuracy         INTEGER,
    pp               INTEGER,
    type_id          INTEGER REFERENCES poke_type(id),
    effect_chance    INTEGER,
    PRIMARY KEY (move_id, version_group_id)
) WITHOUT ROWID;

-- =========================================================
-- LEARNSETS
-- =========================================================
CREATE TABLE form_move (
    form_id          INTEGER NOT NULL REFERENCES pokemon_form(id) ON DELETE CASCADE,
    move_id          INTEGER NOT NULL REFERENCES move_(id),
    version_group_id INTEGER NOT NULL REFERENCES version_group(id),
    method_slug      TEXT NOT NULL,
    level            INTEGER NOT NULL DEFAULT 0,
    machine_slug     TEXT,
    PRIMARY KEY (form_id, move_id, version_group_id, method_slug, level)
) WITHOUT ROWID;
CREATE INDEX idx_formmove_lookup ON form_move(form_id, version_group_id, method_slug);
CREATE INDEX idx_formmove_reverse ON form_move(move_id, version_group_id);

-- =========================================================
-- ÉVOLUTIONS
-- Items are out of the MVP scope, so item references are stored as stable
-- slugs instead of foreign keys.
-- =========================================================
CREATE TABLE evolution_step (
    id                       INTEGER PRIMARY KEY,
    chain_id                 INTEGER NOT NULL REFERENCES evolution_chain(id) ON DELETE CASCADE,
    from_species_id          INTEGER REFERENCES species(id),
    to_species_id            INTEGER NOT NULL REFERENCES species(id),
    trigger_slug             TEXT NOT NULL,
    min_level                INTEGER,
    min_happiness            INTEGER,
    min_affection            INTEGER,
    min_beauty               INTEGER,
    needs_overworld_rain     INTEGER NOT NULL DEFAULT 0,
    time_of_day              TEXT,
    gender                   TEXT,
    held_item_slug           TEXT,
    trigger_item_slug        TEXT,
    known_move_id            INTEGER REFERENCES move_(id),
    known_move_type_id       INTEGER REFERENCES poke_type(id),
    party_species_id         INTEGER REFERENCES species(id),
    party_type_id            INTEGER REFERENCES poke_type(id),
    trade_species_id         INTEGER REFERENCES species(id),
    location_slug            TEXT,
    relative_physical_stats  INTEGER,
    turn_upside_down         INTEGER NOT NULL DEFAULT 0,
    condition_expression     TEXT,
    extra_condition_json     TEXT
);
CREATE INDEX idx_evo_chain ON evolution_step(chain_id);
CREATE INDEX idx_evo_to ON evolution_step(to_species_id);

-- =========================================================
-- LOCALISATION
-- =========================================================
CREATE TABLE translation (
    entity_type TEXT NOT NULL,   -- species, form, move, ability, type, version, stat...
    entity_id   INTEGER NOT NULL,
    lang        TEXT NOT NULL,   -- fr, en
    field       TEXT NOT NULL,   -- name, genus, effect_short, effect_long, description
    value       TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id, lang, field)
) WITHOUT ROWID;
CREATE INDEX idx_trans_lookup ON translation(entity_type, lang, field);

CREATE TABLE flavor_text (
    species_id INTEGER NOT NULL REFERENCES species(id) ON DELETE CASCADE,
    version_id INTEGER NOT NULL REFERENCES game_version(id),
    lang       TEXT NOT NULL,
    text       TEXT NOT NULL,
    PRIMARY KEY (species_id, version_id, lang)
) WITHOUT ROWID;

-- =========================================================
-- MÉDIAS (optional pack, never bundled by default)
-- =========================================================
CREATE TABLE media_asset (
    id          INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id   INTEGER NOT NULL,
    kind        TEXT NOT NULL,   -- sprite_default, sprite_shiny, artwork, icon...
    variant     TEXT,
    rel_path    TEXT NOT NULL,
    width       INTEGER,
    height      INTEGER,
    pack_slug   TEXT
);
CREATE INDEX idx_media_entity ON media_asset(entity_type, entity_id, kind);

-- =========================================================
-- RECHERCHE PLEIN TEXTE
-- =========================================================
CREATE VIRTUAL TABLE search_index USING fts5(
    entity_type     UNINDEXED,
    entity_id       UNINDEXED,
    name,
    name_normalized,
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
